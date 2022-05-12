use crate::library::command;
use std::process;

pub fn go() -> anyhow::Result<()> {
    let out = Out {
        versions: get_versions()?,
    };

    eprintln!("{}", serde_json::to_string_pretty(&out)?);

    Ok(())
}

#[derive(serde::Serialize)]
struct Out {
    versions: Versions,
}

#[derive(serde::Serialize)]
struct Versions {
    git: String,
    kubectl: String,
    skaffold: String,
    ssh: String,
    vagrant: String,
}

fn get_versions() -> anyhow::Result<Versions> {
    Ok(Versions {
        git: command::stdout_utf8(process::Command::new("git").arg("version"))?,
        kubectl: command::stdout_utf8(
            process::Command::new("kubectl")
                .arg("version")
                .arg("--client")
                .arg("--short"),
        )?,
        skaffold: command::stdout_utf8(process::Command::new("skaffold").arg("version"))?,
        ssh: command::stderr_utf8(process::Command::new("ssh").arg("-V"))?,
        vagrant: command::stdout_utf8(process::Command::new("vagrant").arg("--version"))?,
    })
}
