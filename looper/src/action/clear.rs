use crate::library::constants;
use crate::library::run_command;
use std::process;

pub fn go() -> Result<(), String> {
    remove_vm_if_exists()?;

    run_command::go(process::Command::new("git").args(["restore", "."]))?;
    run_command::go(process::Command::new("git").args(["clean", "-d", "--force", "-x"]))
}

fn remove_vm_if_exists() -> Result<(), String> {
    if constants::vagrantfile().exists() {
        run_command::go(
            process::Command::new("vagrant")
                .args(["destroy", "--force"])
                .current_dir(constants::WORK_FOLDER),
        )
    } else {
        Ok(())
    }
}
