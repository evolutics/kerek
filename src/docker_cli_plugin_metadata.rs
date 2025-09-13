pub fn go() -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&Metadata {
        schema_version: "0.1.0".into(), // Update-worthy.
        short_description: env!("CARGO_PKG_DESCRIPTION").into(),
        url: env!("CARGO_PKG_HOMEPAGE").into(),
        vendor: "Benjamin Fischer".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })?)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct Metadata {
    schema_version: String,
    short_description: String,
    #[serde(rename = "URL")]
    url: String,
    vendor: String,
    version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles() -> anyhow::Result<()> {
        let metadata = go()?;
        assert_eq!(
            metadata,
            r#"{
  "SchemaVersion": "0.1.0",
  "ShortDescription": "Light continuous delivery for Docker Compose",
  "URL": "https://github.com/evolutics/kerek",
  "Vendor": "Benjamin Fischer",
  "Version": "2.2.1"
}"#,
        );
        Ok(())
    }
}
