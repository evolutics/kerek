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
    vagrant: String,
}

fn get_versions() -> anyhow::Result<Versions> {
    Ok(Versions {
        vagrant: command::stdout_utf8(process::Command::new("vagrant").arg("--version"))?,
    })
}
