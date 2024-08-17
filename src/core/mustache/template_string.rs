use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Mustache, Segment};
use crate::core::path::PathString;

/// TemplateString acts as wrapper over mustache but supports serialization and
/// deserialization. It provides utilities for parsing, resolving, and comparing
/// template strings.
#[derive(Debug, derive_more::Display, Default, Clone)]
pub struct TemplateString(Mustache);

impl PartialEq for TemplateString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl TryFrom<&str> for TemplateString {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> anyhow::Result<Self> {
        Ok(Self(Mustache::parse(value)))
    }
}

impl TemplateString {
    pub fn is_empty(&self) -> bool {
        self.0.to_string().is_empty()
    }

    pub fn parse(value: &str) -> anyhow::Result<Self> {
        Ok(Self(Mustache::parse(value)))
    }

    pub fn resolve(&self, ctx: &impl PathString) -> Self {
        let resolved_secret = Mustache::from(vec![Segment::Literal(self.0.render(ctx))]);
        Self(resolved_secret)
    }
}

impl Serialize for TemplateString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for TemplateString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let template_string = String::deserialize(deserializer)?;
        let mustache = Mustache::parse(&template_string);

        Ok(TemplateString(mustache))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::core::config::ConfigReaderContext;
    use crate::core::mustache::TemplateString;
    use crate::core::tests::TestEnvIO;
    use crate::core::Mustache;

    #[test]
    fn test_default() {
        let default_template = TemplateString::default();
        assert!(default_template.is_empty());
    }

    #[test]
    fn test_from_str() {
        let template_str = "Hello, World!";
        let template = TemplateString::try_from(template_str).unwrap();
        assert_eq!(template.0.to_string(), template_str);
    }

    #[test]
    fn test_is_empty() {
        let empty_template = TemplateString::default();
        assert!(empty_template.is_empty());

        let non_empty_template = TemplateString::try_from("Hello").unwrap();
        assert!(!non_empty_template.is_empty());
    }

    #[test]
    fn test_parse() {
        let actual = TemplateString::parse("{{.env.TAILCALL_SECRET}}").unwrap();
        let expected = Mustache::parse("{{.env.TAILCALL_SECRET}}").unwrap();
        assert_eq!(actual.0, expected);
    }

    #[test]
    fn test_resolve() {
        let mut env_vars = HashMap::new();
        let token = "eyJhbGciOiJIUzI1NiIsInR5";
        env_vars.insert("TAILCALL_SECRET".to_owned(), token.to_owned());

        let mut runtime = crate::core::runtime::test::init(None);
        runtime.env = Arc::new(TestEnvIO::init(env_vars));

        let ctx = ConfigReaderContext {
            runtime: &runtime,
            vars: &Default::default(),
            headers: Default::default(),
        };

        let actual = TemplateString::parse("{{.env.TAILCALL_SECRET}}")
            .unwrap()
            .resolve(&ctx);
        let expected = TemplateString::try_from("eyJhbGciOiJIUzI1NiIsInR5").unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize() {
        let template = TemplateString::try_from("{{.env.TEST}}").unwrap();
        let serialized = serde_json::to_string(&template).unwrap();
        assert_eq!(serialized, "\"{{env.TEST}}\"");
    }

    #[test]
    fn test_deserialize() {
        let serialized = "\"{{.env.TEST}}\"";
        let template: TemplateString = serde_json::from_str(serialized).unwrap();

        let actual = template.0;
        let expected = Mustache::parse("{{.env.TEST}}").unwrap();

        assert_eq!(actual, expected);
    }
}
