use super::command;
use super::retry;
use std::fs;
use std::path;
use std::process;
use std::time;

pub fn go(in_: In) -> anyhow::Result<()> {
    do_scripted_provisioning(&in_)?;
    reboot(&in_)?;
    test_scripted_provisioning(&in_)?;
    copy_local_kubeconfig(&in_)?;
    adjust_kubeconfig_for_remote_access(&in_)
}

pub struct In<'a> {
    pub script_file: &'a path::Path,
    pub ssh_configuration_file: &'a path::Path,
    pub ssh_host: &'a str,
    pub kubeconfig_file: &'a path::Path,
    pub public_ip: &'a str,
}

fn do_scripted_provisioning(in_: &In) -> anyhow::Result<()> {
    let script = fs::File::open(in_.script_file)?;
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg(in_.ssh_host)
            .arg("--")
            .arg("bash")
            .arg("-s")
            .arg("--")
            .arg("do")
            .stdin(script),
    )
}

fn reboot(in_: &In) -> anyhow::Result<()> {
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg("-f")
            .arg("-l")
            .arg("kerek")
            .arg(in_.ssh_host)
            .arg("--")
            .arg("sudo")
            .arg("reboot"),
    )
}

fn test_scripted_provisioning(in_: &In) -> anyhow::Result<()> {
    retry::go(retry::In {
        total_duration_limit: time::Duration::from_secs(150),
        retry_pause: time::Duration::from_secs(3),
        run: || {
            let script = fs::File::open(in_.script_file)?;
            command::status(
                process::Command::new("ssh")
                    .arg("-F")
                    .arg(in_.ssh_configuration_file)
                    .arg("-l")
                    .arg("kerek")
                    .arg(in_.ssh_host)
                    .arg("--")
                    .arg("bash")
                    .arg("-s")
                    .arg("--")
                    .arg("test")
                    .stdin(script),
            )
        },
    })
}

fn copy_local_kubeconfig(in_: &In) -> anyhow::Result<()> {
    let file = fs::File::create(in_.kubeconfig_file)?;
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg("-l")
            .arg("kerek")
            .arg(in_.ssh_host)
            .arg("--")
            .arg("sudo cat /etc/rancher/k3s/k3s.yaml")
            .stdout(file),
    )
}

fn adjust_kubeconfig_for_remote_access(in_: &In) -> anyhow::Result<()> {
    let public_ip = in_.public_ip;
    command::status(
        process::Command::new("kubectl")
            .arg("--kubeconfig")
            .arg(in_.kubeconfig_file)
            .arg("config")
            .arg("set-cluster")
            .arg("default")
            .arg("--server")
            .arg(format!("https://{public_ip}:6443")),
    )
}
