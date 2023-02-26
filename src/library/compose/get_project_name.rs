use super::interpolated;
use std::collections;
use std::path;

pub fn go(in_: In) -> String {
    [
        get_name_from_override,
        get_name_from_compose_contents,
        get_name_from_folder,
    ]
    .iter()
    .find_map(|get| get(&in_))
    .unwrap_or_else(|| LAST_RESORT_NAME.into())
}

pub struct In<'a> {
    pub compose_contents: &'a str,
    pub compose_file: &'a path::Path,
    pub override_: Option<String>,
    pub variable_overrides: &'a collections::HashMap<String, Option<String>>,
}

const LAST_RESORT_NAME: &str = "default";

#[derive(serde::Deserialize)]
struct Project {
    name: String,
}

fn get_name_from_override(in_: &In) -> Option<String> {
    in_.override_.clone()
}

fn get_name_from_compose_contents(in_: &In) -> Option<String> {
    interpolated::deserialize::<Project>(
        in_.compose_file,
        in_.compose_contents,
        in_.variable_overrides,
    )
    .map(|project| project.name)
    .ok()
}

fn get_name_from_folder(in_: &In) -> Option<String> {
    in_.compose_file
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .map(|name| name.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_0_is_override() {
        assert_eq!(
            go(In {
                compose_contents: "name: a",
                compose_file: path::Path::new("b/compose.yaml"),
                override_: Some("c".into()),
                variable_overrides: &[].into(),
            }),
            "c",
        )
    }

    #[test]
    fn priority_1_is_compose_contents() {
        assert_eq!(
            go(In {
                compose_contents: "name: a",
                compose_file: path::Path::new("b/compose.yaml"),
                override_: None,
                variable_overrides: &[].into(),
            }),
            "a",
        )
    }

    #[test]
    fn priority_2_is_compose_file() {
        assert_eq!(
            go(In {
                compose_contents: "",
                compose_file: path::Path::new("b/compose.yaml"),
                override_: None,
                variable_overrides: &[].into(),
            }),
            "b",
        )
    }

    #[test]
    fn priority_3_is_last_resort() {
        assert_eq!(
            go(In {
                compose_contents: "",
                compose_file: path::Path::new("/compose.yaml"),
                override_: None,
                variable_overrides: &[].into(),
            }),
            "default",
        )
    }
}
