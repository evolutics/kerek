use super::run_bash_script_over_ssh;
use super::run_command;
use std::fs;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    run_scripts(&in_)?;
    dump_kubeconfig(&in_)
}

pub struct In<'a> {
    pub scripts: &'a [String],
    pub ssh_configuration_file: &'a str,
    pub ssh_host: &'a str,
    pub kubeconfig_file: &'a str,
    pub public_ip: &'a str,
}

fn run_scripts(in_: &In) -> anyhow::Result<()> {
    for script in in_.scripts {
        run_bash_script_over_ssh::go(run_bash_script_over_ssh::In {
            configuration_file: in_.ssh_configuration_file,
            host: in_.ssh_host,
            script_file: script,
        })?;
    }
    Ok(())
}

fn dump_kubeconfig(in_: &In) -> anyhow::Result<()> {
    copy_local_kubeconfig(in_)?;
    adjust_kubeconfig_for_remote_access(in_)
}

fn copy_local_kubeconfig(in_: &In) -> anyhow::Result<()> {
    let file = fs::File::create(in_.kubeconfig_file)?;
    run_command::go(
        process::Command::new("ssh")
            .args([
                "-F",
                in_.ssh_configuration_file,
                in_.ssh_host,
                "sudo cat /etc/rancher/k3s/k3s.yaml",
            ])
            .stdout(file),
    )
}

fn adjust_kubeconfig_for_remote_access(in_: &In) -> anyhow::Result<()> {
    let public_ip = in_.public_ip;
    run_command::go(process::Command::new("kubectl").args([
        "--kubeconfig",
        in_.kubeconfig_file,
        "config",
        "set-cluster",
        "default",
        "--server",
        &format!("https://{public_ip}:6443"),
    ]))
}
