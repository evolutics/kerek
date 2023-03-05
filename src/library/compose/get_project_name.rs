use super::interpolated;
use std::path;

pub fn go(in_: In) -> String {
    [
        get_name_from_override,
        get_name_from_compose_contents,
        get_name_from_project_folder,
    ]
    .iter()
    .find_map(|get| get(&in_))
    .unwrap_or_else(|| LAST_RESORT_NAME.into())
}

pub struct In<'a> {
    pub compose_source: &'a interpolated::Source,
    pub override_: Option<String>,
    pub project_folder: &'a path::Path,
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
    interpolated::deserialize::<Project>(in_.compose_source)
        .map(|project| project.name)
        .ok()
}

fn get_name_from_project_folder(in_: &In) -> Option<String> {
    in_.project_folder
        .file_name()
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
                compose_source: &interpolated::Source {
                    contents: "name: a".into(),
                    format: interpolated::Format::Yaml,
                    variable_overrides: [].into(),
                },
                override_: Some("b".into()),
                project_folder: path::Path::new("c"),
            }),
            "b",
        )
    }

    #[test]
    fn priority_1_is_compose_contents() {
        assert_eq!(
            go(In {
                compose_source: &interpolated::Source {
                    contents: "name: a".into(),
                    format: interpolated::Format::Yaml,
                    variable_overrides: [].into(),
                },
                override_: None,
                project_folder: path::Path::new("c"),
            }),
            "a",
        )
    }

    #[test]
    fn priority_2_is_project_folder() {
        assert_eq!(
            go(In {
                compose_source: &interpolated::Source {
                    contents: "".into(),
                    format: interpolated::Format::Yaml,
                    variable_overrides: [].into(),
                },
                override_: None,
                project_folder: path::Path::new("c"),
            }),
            "c",
        )
    }

    #[test]
    fn priority_3_is_last_resort() {
        assert_eq!(
            go(In {
                compose_source: &interpolated::Source {
                    contents: "".into(),
                    format: interpolated::Format::Yaml,
                    variable_overrides: [].into(),
                },
                override_: None,
                project_folder: path::Path::new(""),
            }),
            "default",
        )
    }
}
