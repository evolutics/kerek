use std::process;

pub fn go(command: &mut process::Command) -> Result<(), String> {
    match command.status() {
        Err(error) => Err(format!("{command:?}: {error}")),
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("{command:?}: {status}"))
            }
        }
    }
}
