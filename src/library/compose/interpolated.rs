use super::interpolate;
use serde::de;
use std::collections;
use std::path;

pub fn deserialize<T: de::DeserializeOwned>(
    file: &path::Path,
    contents: &str,
    extra_variables: &collections::HashMap<String, Option<String>>,
) -> anyhow::Result<T> {
    let value = match file.extension() {
        Some(extension) if extension == "toml" => toml::from_str(contents)?,
        _ => serde_yaml::from_str(contents)?,
    };

    let value = map_string_values(value, |string| {
        interpolate::go(&string, extra_variables).map(|string| string.into())
    })?;

    serde_path_to_error::deserialize(value).map_err(|error| anyhow::anyhow!("{error}"))
}

pub fn serialize<T: serde::Serialize>(value: T) -> anyhow::Result<String> {
    let value = serde_yaml::to_value(value)?;

    let value = map_string_values(value, |string| Ok(string.replace('$', "$$")))?;

    Ok(serde_yaml::to_string(&value)?)
}

fn map_string_values<F: Copy + Fn(String) -> anyhow::Result<String>>(
    value: serde_yaml::Value,
    function: F,
) -> anyhow::Result<serde_yaml::Value> {
    Ok(match value {
        serde_yaml::Value::Mapping(mapping) => serde_yaml::Value::Mapping(
            mapping
                .into_iter()
                .map(|(key, value)| map_string_values(value, function).map(|value| (key, value)))
                .collect::<Result<_, _>>()?,
        ),
        serde_yaml::Value::Sequence(sequence) => serde_yaml::Value::Sequence(
            sequence
                .into_iter()
                .map(|value| map_string_values(value, function))
                .collect::<Result<_, _>>()?,
        ),
        serde_yaml::Value::String(string) => serde_yaml::Value::String(function(string)?),
        value => value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes() -> anyhow::Result<()> {
        assert_eq!(
            deserialize::<Container>(
                path::Path::new(""),
                "field: '${SOME} days'",
                &[("SOME".into(), Some("X".into()))].into(),
            )?,
            Container {
                field: "X days".into(),
            },
        );
        Ok(())
    }

    #[test]
    fn serializes() -> anyhow::Result<()> {
        assert_eq!(
            serialize(Container {
                field: "$5 $$".into(),
            })?,
            "field: $$5 $$$$\n",
        );
        Ok(())
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct Container {
        field: String,
    }
}
