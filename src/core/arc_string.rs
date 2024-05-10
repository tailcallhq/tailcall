use std::sync::Arc;

use headers::HeaderValue;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcString(Arc<str>);

impl<'de> Deserialize<'de> for ArcString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(ArcString::from)
    }
}

impl Serialize for ArcString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl Default for ArcString {
    fn default() -> Self {
        ArcString("".into())
    }
}

impl AsRef<str> for ArcString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ArcString {
    fn from(s: &str) -> Self {
        ArcString(s.into())
    }
}

impl From<&String> for ArcString {
    fn from(s: &String) -> Self {
        ArcString(s.clone().into())
    }
}

impl From<String> for ArcString {
    fn from(s: String) -> Self {
        ArcString(s.into())
    }
}

impl From<ArcString> for Vec<u8> {
    fn from(s: ArcString) -> Self {
        s.as_str().as_bytes().to_vec()
    }
}

impl TryFrom<ArcString> for HeaderValue {
    type Error = <HeaderValue as TryFrom<String>>::Error;

    fn try_from(s: ArcString) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}

impl PartialEq<&str> for ArcString {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl schemars::JsonSchema for ArcString {
    fn schema_name() -> String {
        <String as schemars::JsonSchema>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <String as schemars::JsonSchema>::json_schema(gen)
    }
}

impl std::fmt::Display for ArcString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl ArcString {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
