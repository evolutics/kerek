use super::interpolate;
use serde::de;
use std::fmt;

#[derive(Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StrBuf(String);

impl From<String> for StrBuf {
    fn from(string: String) -> Self {
        Self(string)
    }
}

impl From<StrBuf> for String {
    fn from(string: StrBuf) -> Self {
        string.0
    }
}

impl From<&str> for StrBuf {
    fn from(string: &str) -> Self {
        Self(string.into())
    }
}

impl<'d> serde::Deserialize<'d> for StrBuf {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(StrBufVisitor)
    }
}

struct StrBufVisitor;

impl<'d> de::Visitor<'d> for StrBufVisitor {
    type Value = StrBuf;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match interpolate::go(value) {
            // TODO: Improve error reporting.
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(value), &self)),
            Ok(value) => Ok(StrBuf(value.into())),
        }
    }
}

impl serde::Serialize for StrBuf {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.replace('$', "$$"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn deserializes_string() -> anyhow::Result<()> {
        env::set_var("WHEELSTICKS_SOME", "X");

        assert_eq!(
            serde_yaml::from_str::<StrBuf>("${WHEELSTICKS_SOME} days")?,
            "X days".into(),
        );
        Ok(())
    }

    #[test]
    fn serializes_string() -> anyhow::Result<()> {
        assert_eq!(serde_yaml::to_string(&StrBuf::from("$5 $$"))?, "$$5 $$$$\n");
        Ok(())
    }
}
