use super::command;
use super::run_bash_script_over_ssh;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    run_script(&in_)?;
    dump_kubeconfig(&in_)
}

pub struct In<'a> {
    pub script_file: &'a path::Path,
    pub ssh_configuration_file: &'a path::Path,
    pub ssh_host: &'a str,
    pub kubeconfig_file: &'a path::Path,
    pub public_ip: &'a str,
}

fn run_script(in_: &In) -> anyhow::Result<()> {
    run_bash_script_over_ssh::go(run_bash_script_over_ssh::In {
        configuration_file: in_.ssh_configuration_file,
        host: in_.ssh_host,
        script_file: in_.script_file,
    })
}

fn dump_kubeconfig(in_: &In) -> anyhow::Result<()> {
    copy_local_kubeconfig(in_)?;
    adjust_kubeconfig_for_remote_access(in_)
}

fn copy_local_kubeconfig(in_: &In) -> anyhow::Result<()> {
    let file = fs::File::create(in_.kubeconfig_file)?;
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg(in_.ssh_host)
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
