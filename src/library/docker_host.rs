use super::command;
use anyhow::Context;
use std::env;
use std::process;

#[derive(Debug, PartialEq)]
pub struct Host {
    pub hostname: String,
    pub port: Option<u16>,
    pub scheme: Scheme,
    pub url: String,
    pub user: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Scheme {
    Other,
    Ssh,
}

pub fn get(url_override: Option<String>) -> anyhow::Result<Host> {
    let effective_url = get_effective_url(url_override)?;

    let url = url::Url::parse(&effective_url)
        .with_context(|| format!("Unable to parse Docker host URL {effective_url:?}"))?;

    Ok(Host {
        hostname: url.host_str().unwrap_or("").into(),
        port: url.port(),
        scheme: if url.scheme() == "ssh" {
            Scheme::Ssh
        } else {
            Scheme::Other
        },
        url: effective_url,
        user: (!url.username().is_empty()).then(|| url.username().into()),
    })
}

fn get_effective_url(url_override: Option<String>) -> anyhow::Result<String> {
    match url_override {
        None => match env::var(ENVIRONMENT_VARIABLE) {
            Err(env::VarError::NotPresent) => {
                let context = get_current_context().with_context(|| {
                    format!(
                        "Unable to get current Docker context, \
                        try using {ENVIRONMENT_VARIABLE:?} instead"
                    )
                })?;
                Ok(context.endpoints.docker.host)
            }
            Err(env::VarError::NotUnicode(url)) => Err(anyhow::anyhow!(
                "Environment variable {ENVIRONMENT_VARIABLE:?} \
                should be Unicode but is {url:?}"
            )),
            Ok(url) => Ok(url),
        },
        Some(url) => Ok(url),
    }
}

const ENVIRONMENT_VARIABLE: &str = "DOCKER_HOST";

fn get_current_context() -> anyhow::Result<DockerContext> {
    command::stdout_json(process::Command::new("docker").args([
        "context",
        "inspect",
        "--format",
        "{{json .}}",
    ]))
}

#[derive(serde::Deserialize)]
struct DockerContext {
    #[serde(rename = "Endpoints")]
    endpoints: Endpoints,
}

#[derive(serde::Deserialize)]
struct Endpoints {
    docker: Endpoint,
}

#[derive(serde::Deserialize)]
struct Endpoint {
    #[serde(rename = "Host")]
    host: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get {
        use super::*;

        #[test]
        fn handles_invalid_url() {
            assert!(get(Some("..".into())).is_err())
        }

        #[test]
        fn handles_ssh_url() -> anyhow::Result<()> {
            assert_eq!(
                get(Some("ssh://abc@example.com:123".into()))?,
                Host {
                    hostname: "example.com".into(),
                    port: Some(123),
                    scheme: Scheme::Ssh,
                    url: "ssh://abc@example.com:123".into(),
                    user: Some("abc".into()),
                },
            );
            Ok(())
        }

        #[test]
        fn handles_other_url() -> anyhow::Result<()> {
            assert_eq!(
                get(Some("unix:///tmp/a.sock".into()))?,
                Host {
                    hostname: "".into(),
                    port: None,
                    scheme: Scheme::Other,
                    url: "unix:///tmp/a.sock".into(),
                    user: None,
                },
            );
            Ok(())
        }
    }
}
