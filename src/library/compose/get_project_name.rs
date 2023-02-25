pub fn go(in_: In) -> String {
    [get_name_from_override, get_name_from_compose_contents]
        .iter()
        .find_map(|get| get(&in_))
        .unwrap_or_else(|| LAST_RESORT_NAME.into())
}

pub struct In<'a> {
    pub compose_contents: &'a str,
    pub override_: Option<String>,
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
    serde_yaml::from_str::<Project>(in_.compose_contents)
        .map(|project| project.name)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_0_is_override() {
        assert_eq!(
            go(In {
                compose_contents: "name: a",
                override_: Some("b".into()),
            }),
            "b",
        )
    }

    #[test]
    fn priority_1_is_compose_contents() {
        assert_eq!(
            go(In {
                compose_contents: "name: a",
                override_: None,
            }),
            "a",
        )
    }

    #[test]
    fn priority_2_is_last_resort() {
        assert_eq!(
            go(In {
                compose_contents: "",
                override_: None,
            }),
            "default",
        )
    }
}
