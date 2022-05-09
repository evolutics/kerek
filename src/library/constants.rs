pub const CONFIGURATION_FILE: &str = "kerek.json";

pub const PROVISION_BASE: &str = include_str!("provision_base.sh");

pub const STAGING_IP: &str = "192.168.63.63";

pub const VAGRANTFILE: &str = include_str!("Vagrantfile");

pub const VM_NAME: &str = "default";

pub const WORK_FOLDER: &str = ".kerek";

pub fn provision_base_file() -> String {
    format!("{WORK_FOLDER}/provision_base.sh")
}

pub fn ssh_configuration_file() -> String {
    format!("{WORK_FOLDER}/ssh_configuration")
}

pub fn staging_kubeconfig_file() -> String {
    format!("{WORK_FOLDER}/staging_kubeconfig")
}

pub fn vagrantfile_file() -> String {
    format!("{WORK_FOLDER}/Vagrantfile")
}
