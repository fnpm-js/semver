use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use smol_str::SmolStr;

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagSpec(SmolStr);

impl TagSpec {
    pub fn parse(input: &str) -> Result<Self> {
        let tag = input.trim();
        if tag.is_empty()
            || tag.starts_with('.')
            || tag.contains('/')
            || tag.contains(':')
            || tag.chars().any(char::is_whitespace)
            || tag == "*"
        {
            return Err(Error::InvalidTag {
                input: input.to_string(),
            });
        }

        Ok(Self(SmolStr::new(tag)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TagSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl Serialize for TagSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TagSpec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}
