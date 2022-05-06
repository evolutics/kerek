use std::process;

#[allow(dead_code)]
pub fn go(command: &mut process::Command) -> Result<(), String> {
    match command.status() {
        Result::Err(error) => Result::Err(format!("{command:?}: {error}")),
        Result::Ok(status) => {
            if status.success() {
                Result::Ok(())
            } else {
                Result::Err(format!("{command:?}: {status}"))
            }
        }
    }
}
