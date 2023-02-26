use std::collections;

pub fn go(environment_file: &str) -> collections::HashMap<String, Option<String>> {
    environment_file
        .lines()
        .flat_map(|line| {
            (!line.starts_with('#') && !line.is_empty()).then(|| match line.split_once('=') {
                None => (line.into(), None),
                Some((key, value)) => (key.into(), Some(value.into())),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case("", &[]; "blank")]
    #[test_case::test_case("#X=Y", &[]; "comment")]
    #[test_case::test_case("VAR", &[("VAR", None)]; "unset")]
    #[test_case::test_case("VAR=", &[("VAR", Some(""))]; "empty")]
    #[test_case::test_case("VAR=\"quoted\"", &[("VAR", Some("\"quoted\""))]; "quoted")]
    #[test_case::test_case("VAR=VAL", &[("VAR", Some("VAL"))]; "nonempty")]
    #[test_case::test_case("X\n#\nY=Z\n", &[("X", None), ("Y", Some("Z"))]; "many")]
    fn handles(environment_file: &str, expected: &[(&str, Option<&str>)]) -> anyhow::Result<()> {
        let expected = expected
            .iter()
            .map(|(key, value)| ((*key).into(), value.map(|value| value.into())))
            .collect::<collections::HashMap<_, _>>();

        assert_eq!(go(environment_file), expected);
        Ok(())
    }
}
