use crate::library::command;
use crate::library::compose;
use std::collections;
use std::fs;
use std::path;
use std::process;
use std::thread;
use std::time;

pub fn go(project: &compose::Project) -> anyhow::Result<()> {
    let target_image_ids = load_target_images(project)?;
    let images = get_images()?;

    let actual_images = images
        .iter()
        .filter(|image| image.container_count != 0)
        .collect();
    let target_images = images
        .iter()
        .filter(|image| target_image_ids.contains(&image.image_id))
        .collect();

    let changes = plan_changes(actual_images, target_images);

    for change in changes {
        apply_change(&change)?;
    }
    collect_garbage()
}

const USER_SYSTEMD_FOLDER: &str = ".config/systemd/user";

#[derive(Eq, Hash, PartialEq)]
struct Image {
    container_count: u16,
    container_names: Vec<String>,
    digest: String,
    health_check: Option<String>,
    image_id: String,
    networks: Vec<String>,
    port_mappings: Vec<String>,
    volume_mounts: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawImage {
    pub containers: u16,
    pub digest: String,
    pub id: String,
    pub labels: collections::HashMap<String, String>,
}

#[derive(Eq, Hash, PartialEq)]
struct ContainerChange {
    container_name: String,
    health_check: Option<String>,
    image_digest: String,
    image_id: String,
    networks: Vec<String>,
    operator: Operator,
    port_mappings: Vec<String>,
    systemd_unit: String,
    volume_mounts: Vec<String>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum Operator {
    Add,
    Keep,
    Remove,
}

fn load_target_images(project: &compose::Project) -> anyhow::Result<collections::HashSet<String>> {
    let remote_workbench = &project.x_wheelsticks.remote_workbench;
    let image_files = fs::read_dir(remote_workbench)?
        .into_iter()
        .map(|result| result.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| path.extension() == Some("tar".as_ref()))
        .collect::<collections::BTreeSet<_>>();
    for image_file in image_files.iter() {
        println!("Loading image file {image_file:?}.");
        command::status_ok(
            process::Command::new("podman")
                .args(["load", "--input"])
                .arg(image_file),
        )?;
    }
    Ok(image_files
        .iter()
        .flat_map(|image_file| {
            image_file
                .file_stem()
                .map(|image_id| image_id.to_string_lossy().into())
        })
        .collect())
}

fn get_images() -> anyhow::Result<collections::HashSet<Image>> {
    let images = serde_json::from_slice::<Vec<_>>(&command::stdout_raw(
        process::Command::new("podman").args(["images", "--format", "json"]),
    )?)?;
    Ok(images.into_iter().map(parse_image_metadata).collect())
}

fn parse_image_metadata(image: RawImage) -> Image {
    let labels = image.labels;
    Image {
        container_count: image.containers,
        container_names: csv_fields(labels.get("info.evolutics.wheelsticks.container-names")),
        digest: image.digest,
        health_check: labels
            .get("info.evolutics.wheelsticks.health-check")
            .cloned(),
        image_id: image.id,
        networks: csv_fields(labels.get("info.evolutics.wheelsticks.networks")),
        port_mappings: csv_fields(labels.get("info.evolutics.wheelsticks.port-mappings")),
        volume_mounts: csv_fields(labels.get("info.evolutics.wheelsticks.volume-mounts")),
    }
}

fn csv_fields(optional_string: Option<&String>) -> Vec<String> {
    match optional_string {
        None => vec![],
        Some(string) => string.split(',').map(|field| field.into()).collect(),
    }
}

fn plan_changes(
    actual_images: collections::HashSet<&Image>,
    target_images: collections::HashSet<&Image>,
) -> Vec<ContainerChange> {
    let mut changes = [
        (Operator::Remove, actual_images),
        (Operator::Add, target_images),
    ]
    .iter()
    .flat_map(|(operator, images)| {
        images.iter().flat_map(|image| {
            image
                .container_names
                .iter()
                .map(|container_name| ContainerChange {
                    container_name: container_name.clone(),
                    health_check: image.health_check.clone(),
                    image_digest: image.digest.clone(),
                    image_id: image.image_id.clone(),
                    networks: image.networks.clone(),
                    operator: operator.clone(),
                    port_mappings: image.port_mappings.clone(),
                    systemd_unit: format!("container-{container_name}.service"),
                    volume_mounts: image.volume_mounts.clone(),
                })
        })
    })
    .collect::<Vec<_>>();

    // Reasons to order changes (stable sort):
    // 1. Zero-downtime deployments are possible if there are multiple replicas:
    //    while a container `x-0` is replaced, a load balancer can still forward
    //    traffic to a replica `x-1`; at any time, either replica is available.
    // 2. The following simplification is easier.
    // 3. Clear predictability.
    changes.sort_by(|a, b| a.container_name.cmp(&b.container_name));
    let changes = changes;

    let mut previous_changes = vec![];
    for next_change in changes {
        match previous_changes.last_mut() {
            Some(
                previous_change @ ContainerChange {
                    operator: Operator::Remove,
                    ..
                },
            ) if next_change.operator == Operator::Add
                && previous_change.container_name == next_change.container_name
                && previous_change.image_digest == next_change.image_digest =>
            // Other relevant fields are captured by comparing the image digest.
            {
                previous_change.operator = Operator::Keep
            }
            _ => previous_changes.push(next_change),
        }
    }

    previous_changes
}

fn apply_change(change: &ContainerChange) -> anyhow::Result<()> {
    let container_name = &change.container_name;
    let image_digest = &change.image_digest;
    let operand = format!("container {container_name:?} of image {image_digest:?}");

    match change.operator {
        Operator::Add => {
            println!("Adding {operand}.");
            add_container(change)
        }
        Operator::Keep => {
            println!("Keeping {operand}.");
            Ok(())
        }
        Operator::Remove => {
            println!("Removing {operand}.");
            remove_container(change)
        }
    }
}

fn add_container(change: &ContainerChange) -> anyhow::Result<()> {
    for network in &change.networks {
        create_network_if_not_exists(network)?;
    }

    command::status_ok(
        process::Command::new("podman")
            .arg("create")
            .args(
                change
                    .health_check
                    .iter()
                    .flat_map(|health_check| ["--health-cmd", health_check]),
            )
            .args(["--name", &change.container_name])
            .args(
                change
                    .networks
                    .iter()
                    .flat_map(|network| ["--network", network]),
            )
            .args(
                change
                    .port_mappings
                    .iter()
                    .flat_map(|port_mapping| ["--publish", port_mapping]),
            )
            .args(
                change
                    .volume_mounts
                    .iter()
                    .flat_map(|volume_mount| ["--volume", volume_mount]),
            )
            .args(["--", &change.image_id]),
    )?;
    command::status_ok(
        process::Command::new("podman")
            .args([
                "generate",
                "systemd",
                "--files",
                "--name",
                "--restart-policy",
                "always",
                "--",
                &change.container_name,
            ])
            .current_dir(USER_SYSTEMD_FOLDER),
    )?;
    command::status_ok(process::Command::new("systemctl").args([
        "--now",
        "--user",
        "enable",
        &change.systemd_unit,
    ]))?;

    let mut timeout = time::Duration::from_secs(5);
    while change.health_check.is_some() {
        match command::status_within_time(
            process::Command::new("podman").args(["healthcheck", "run", &change.container_name]),
            timeout,
        )? {
            command::StatusWithinTime::Failure => (),
            command::StatusWithinTime::Success => break,
            command::StatusWithinTime::Timeout => timeout *= 2,
        }
        thread::sleep(timeout)
    }

    Ok(())
}

fn create_network_if_not_exists(network: &str) -> anyhow::Result<()> {
    if command::status_bit(
        process::Command::new("podman").args(["network", "exists", "--", network]),
    )? {
        println!("Creating network {network:?}.");
        command::status_ok(
            process::Command::new("podman").args(["network", "create", "--", network]),
        )?;
    }

    Ok(())
}

fn remove_container(change: &ContainerChange) -> anyhow::Result<()> {
    command::status_ok(process::Command::new("systemctl").args([
        "--now",
        "--user",
        "disable",
        &change.systemd_unit,
    ]))?;
    fs::remove_file(path::Path::new(USER_SYSTEMD_FOLDER).join(&change.systemd_unit))?;
    command::status_ok(process::Command::new("podman").args(["rm", "--", &change.container_name]))
}

fn collect_garbage() -> anyhow::Result<()> {
    println!("Collecting garbage.");
    command::status_ok(process::Command::new("podman").args([
        "system",
        "prune",
        "--all",
        "--force",
        "--volumes",
    ]))
}
