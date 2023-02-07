#!/usr/bin/env python3

import csv
import dataclasses
import datetime
import enum
import functools
import io
import json
import os
import pathlib
import subprocess
import time


def main():
    target_image_ids = _load_target_images()
    images = _get_images()

    actual_images = {image for image in images if image.container_count != 0}
    target_images = {image for image in images if image.image_id in target_image_ids}

    changes = _plan_changes(actual_images, target_images)

    for change in changes:
        _apply_change(change)
    _collect_garbage()


_USER_SYSTEMD_FOLDER = pathlib.Path(".config") / "systemd" / "user"


@dataclasses.dataclass(frozen=True)
class _Image:
    container_count: int
    container_names: tuple[str, ...]
    digest: str
    health_check: str | None
    image_id: str
    networks: tuple[str, ...]
    port_mappings: tuple[str, ...]
    volume_mounts: tuple[str, ...]


_Operator = enum.Enum("_Operator", ["ADD", "KEEP", "REMOVE"])


@dataclasses.dataclass
class _ContainerChange:
    container_name: str
    health_check: str | None
    image_digest: str
    image_id: str
    networks: tuple[str, ...]
    operator: _Operator
    port_mappings: tuple[str, ...]
    systemd_unit: str
    volume_mounts: tuple[str, ...]


def _load_target_images():
    remote_workbench = pathlib.Path(os.environ["WHEELSTICKS_REMOTE_WORKBENCH"])
    image_files = sorted(remote_workbench.glob("*.tar"))
    for image_file in image_files:
        print(f"Loading image file {str(image_file)!r}.")
        subprocess.run(["podman", "load", "--input", image_file], check=True)
    return {image_file.stem for image_file in image_files}


def _get_images():
    images = json.loads(
        subprocess.run(
            ["podman", "images", "--format", "json"], check=True, stdout=subprocess.PIPE
        ).stdout
    )
    return {_parse_image_metadata(image) for image in images}


def _parse_image_metadata(image):
    labels = image["Labels"]
    return _Image(
        container_count=int(image["Containers"]),
        container_names=_csv_fields(
            labels.get("info.evolutics.wheelsticks.container-names")
        ),
        digest=image["Digest"],
        health_check=labels.get("info.evolutics.wheelsticks.health-check"),
        image_id=image["Id"],
        networks=_csv_fields(labels.get("info.evolutics.wheelsticks.networks")),
        port_mappings=_csv_fields(
            labels.get("info.evolutics.wheelsticks.port-mappings")
        ),
        volume_mounts=_csv_fields(
            labels.get("info.evolutics.wheelsticks.volume-mounts")
        ),
    )


def _csv_fields(optional_string):
    string = "" if optional_string is None else optional_string
    records = csv.reader(io.StringIO(string))
    return tuple(field for record in records for field in record)


def _plan_changes(actual_images, target_images):
    changes = [
        _ContainerChange(
            container_name=container_name,
            health_check=image.health_check,
            image_digest=image.digest,
            image_id=image.image_id,
            networks=image.networks,
            operator=operator,
            port_mappings=image.port_mappings,
            systemd_unit=f"container-{container_name}.service",
            volume_mounts=image.volume_mounts,
        )
        for operator, images in (
            (_Operator.REMOVE, actual_images),
            (_Operator.ADD, target_images),
        )
        for image in images
        for container_name in image.container_names
    ]

    # Reasons to order changes (stable sort):
    # 1. Zero-downtime deployments are possible if there are multiple replicas:
    #    while a container `x-0` is replaced, a load balancer can still forward
    #    traffic to a replica `x-1`; at any time, either replica is available.
    # 2. The following simplification is easier.
    # 3. Clear predictability.
    changes.sort(key=lambda change: change.container_name)

    def cancel_removal_followed_by_addition(previous_changes, next_change):
        if (
            previous_changes
            and previous_changes[-1].operator == _Operator.REMOVE
            and next_change.operator == _Operator.ADD
            and previous_changes[-1].container_name == next_change.container_name
            and previous_changes[-1].image_digest == next_change.image_digest
            # Other relevant fields are captured by comparing the image digest.
        ):
            result = dataclasses.replace(previous_changes[-1], operator=_Operator.KEEP)
            return previous_changes[:-1] + [result]
        return previous_changes + [next_change]

    changes = functools.reduce(cancel_removal_followed_by_addition, changes, [])

    return changes


def _apply_change(change):
    operand = f"container {change.container_name!r} of image {change.image_digest!r}"

    summary, operation = {
        _Operator.ADD: (f"Adding {operand}.", _add_container),
        _Operator.KEEP: (f"Keeping {operand}.", lambda _: None),
        _Operator.REMOVE: (f"Removing {operand}.", _remove_container),
    }[change.operator]

    print(summary)
    operation(change)


def _add_container(change):
    for network in change.networks:
        _create_network_if_not_exists(network)

    subprocess.run(
        ["podman", "create"]
        + ([f"--health-cmd={change.health_check}"] if change.health_check else [])
        + ["--name", change.container_name]
        + [f"--network={network}" for network in change.networks]
        + [f"--publish={port_mapping}" for port_mapping in change.port_mappings]
        + [f"--volume={volume_mount}" for volume_mount in change.volume_mounts]
        + ["--", change.image_id],
        check=True,
    )
    subprocess.run(
        [
            "podman",
            "generate",
            "systemd",
            "--files",
            "--name",
            "--restart-policy",
            "always",
            "--",
            change.container_name,
        ],
        check=True,
        cwd=_USER_SYSTEMD_FOLDER,
    )
    subprocess.run(
        ["systemctl", "--now", "--user", "enable", change.systemd_unit], check=True
    )

    timeout = datetime.timedelta(seconds=5)
    while change.health_check:
        try:
            subprocess.run(
                ["podman", "healthcheck", "run", change.container_name],
                check=True,
                timeout=timeout.total_seconds(),
            )
        except subprocess.CalledProcessError:
            pass
        except subprocess.TimeoutExpired:
            timeout *= 2
        else:
            break
        time.sleep(timeout.total_seconds())


def _create_network_if_not_exists(network):
    try:
        subprocess.run(["podman", "network", "exists", "--", network], check=True)
    except subprocess.CalledProcessError as error:
        if error.returncode == 1:
            print(f"Creating network {network!r}.")
            subprocess.run(["podman", "network", "create", "--", network], check=True)
        else:
            raise


def _remove_container(change):
    subprocess.run(
        ["systemctl", "--now", "--user", "disable", change.systemd_unit], check=True
    )
    (_USER_SYSTEMD_FOLDER / change.systemd_unit).unlink()
    subprocess.run(["podman", "rm", "--", change.container_name], check=True)


def _collect_garbage():
    print("Collecting garbage.")
    subprocess.run(
        ["podman", "system", "prune", "--all", "--force", "--volumes"], check=True
    )


if __name__ == "__main__":
    main()
