use std::collections::BTreeMap;
use std::env;
use std::marker::PhantomData;
use std::path::Path;

use derive_setters::Setters;
use path_clean::PathClean;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::core::config::transformer::Preset;
use crate::core::config::{self, ConfigReaderContext};
use crate::core::http::Method;
use crate::core::mustache::TemplateString;
use crate::core::valid::{Valid, ValidateFrom, Validator};

#[derive(Deserialize, Serialize, Debug, Default, Setters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Config<Status = UnResolved> {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input<Status>>,
    pub output: Output<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<PresetConfig>,
    pub schema: Schema,
    #[serde(default, skip_serializing_if = "TemplateString::is_empty")]
    pub secret: TemplateString,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PresetConfig {
    pub merge_type: Option<f32>,
    #[serde(rename = "consolidateURL")]
    pub consolidate_url: Option<f32>,
    pub infer_type_names: Option<bool>,
    pub tree_shake: Option<bool>,
    pub unwrap_single_field_types: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(transparent)]
pub struct Location<A>(
    #[serde(skip_serializing_if = "Location::is_empty")] pub String,
    #[serde(skip)] PhantomData<A>,
);

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Headers<A>(
    #[serde(skip_serializing_if = "is_default")] Option<BTreeMap<String, TemplateString>>,
    #[serde(skip)] PhantomData<A>,
);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Input<Status = UnResolved> {
    #[serde(flatten)]
    pub source: Source<Status>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Source<Status = UnResolved> {
    #[serde(rename_all = "camelCase")]
    Curl {
        src: Location<Status>,
        headers: Headers<Status>,
        #[serde(skip_serializing_if = "Option::is_none")]
        method: Option<Method>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_mutation: Option<bool>,
        field_name: String,
    },
    Proto {
        src: Location<Status>,
    },
    Config {
        src: Location<Status>,
    },
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Output<Status = UnResolved> {
    #[serde(skip_serializing_if = "Location::is_empty")]
    pub path: Location<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<config::Source>,
}

#[derive(Debug)]
pub enum Resolved {}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnResolved {}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<String>,
}

fn between(threshold: f32, min: f32, max: f32) -> Valid<(), String> {
    Valid::<(), String>::fail(format!(
        "Invalid threshold value ({:.2}). Allowed range is [{:.2} - {:.2}] inclusive.",
        threshold, min, max
    ))
    .when(|| !(min..=max).contains(&threshold))
}

impl ValidateFrom<PresetConfig> for Preset {
    type Error = String;
    fn validate_from(config: PresetConfig) -> Valid<Self, Self::Error> {
        let mut preset = Preset::new();

        if let Some(merge_type) = config.merge_type {
            preset = preset.merge_type(merge_type);
        }

        if let Some(consolidate_url) = config.consolidate_url {
            preset = preset.consolidate_url(consolidate_url);
        }

        if let Some(use_better_names) = config.infer_type_names {
            preset = preset.infer_type_names(use_better_names);
        }

        if let Some(unwrap_single_field_types) = config.unwrap_single_field_types {
            preset = preset.unwrap_single_field_types(unwrap_single_field_types);
        }

        if let Some(tree_shake) = config.tree_shake {
            preset = preset.tree_shake(tree_shake);
        }

        // TODO: The field names in trace should be inserted at compile time.
        Valid::succeed(preset)
            .and_then(|preset| {
                let merge_types_th = between(preset.merge_type, 0.0, 1.0).trace("mergeType");
                let consolidate_url_th =
                    between(preset.consolidate_url, 0.0, 1.0).trace("consolidateURL");

                merge_types_th.and(consolidate_url_th).map_to(preset)
            })
            .trace("preset")
    }
}

impl<A> Location<A> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Location<UnResolved> {
    fn into_resolved(self, parent_dir: Option<&Path>) -> Location<Resolved> {
        let path = {
            let path = self.0.as_str();
            if Url::parse(path).is_ok() || Path::new(path).is_absolute() {
                path.to_string()
            } else {
                let parent_dir = parent_dir.unwrap_or(Path::new(""));
                let joined_path = parent_dir.join(path);
                if let Ok(abs_path) = std::fs::canonicalize(&joined_path) {
                    abs_path.to_string_lossy().to_string()
                } else if let Ok(cwd) = env::current_dir() {
                    cwd.join(joined_path).clean().to_string_lossy().to_string()
                } else {
                    joined_path.clean().to_string_lossy().to_string()
                }
            }
        };
        Location(path, PhantomData)
    }
}

impl<A> Headers<A> {
    pub fn as_btree_map(&self) -> &Option<BTreeMap<String, TemplateString>> {
        &self.0
    }
}

impl Headers<UnResolved> {
    pub fn resolve(self, reader_context: &ConfigReaderContext) -> Headers<Resolved> {
        // Resolve the header values with mustache template.
        let resolved_headers = self.0.map(|headers_inner| {
            headers_inner
                .into_iter()
                .map(|(k, v)| (k, v.resolve(reader_context)))
                .collect::<BTreeMap<_, _>>()
        });

        Headers(resolved_headers, PhantomData)
    }
}

impl Output<UnResolved> {
    pub fn resolve(self, parent_dir: Option<&Path>) -> anyhow::Result<Output<Resolved>> {
        Ok(Output {
            format: self.format,
            path: self.path.into_resolved(parent_dir),
        })
    }
}

impl Source<UnResolved> {
    pub fn resolve(
        self,
        parent_dir: Option<&Path>,
        reader_context: &ConfigReaderContext,
    ) -> anyhow::Result<Source<Resolved>> {
        match self {
            Source::Curl { src, field_name, headers, body, method, is_mutation } => {
                let resolved_path = src.into_resolved(parent_dir);
                let resolved_headers = headers.resolve(reader_context);
                Ok(Source::Curl {
                    src: resolved_path,
                    field_name,
                    headers: resolved_headers,
                    body,
                    method,
                    is_mutation,
                })
            }
            Source::Proto { src } => {
                let resolved_path = src.into_resolved(parent_dir);
                Ok(Source::Proto { src: resolved_path })
            }
            Source::Config { src } => {
                let resolved_path = src.into_resolved(parent_dir);
                Ok(Source::Config { src: resolved_path })
            }
        }
    }
}

impl Input<UnResolved> {
    pub fn resolve(
        self,
        parent_dir: Option<&Path>,
        reader_context: &ConfigReaderContext,
    ) -> anyhow::Result<Input<Resolved>> {
        let resolved_source = self.source.resolve(parent_dir, reader_context)?;
        Ok(Input { source: resolved_source })
    }
}

impl Config {
    /// Resolves all the relative paths present inside the GeneratorConfig.
    pub fn into_resolved(
        self,
        config_path: &str,
        reader_context: ConfigReaderContext,
    ) -> anyhow::Result<Config<Resolved>> {
        let parent_dir = Some(Path::new(config_path).parent().unwrap_or(Path::new("")));

        let inputs = self
            .inputs
            .into_iter()
            .map(|input| input.resolve(parent_dir, &reader_context))
            .collect::<anyhow::Result<Vec<Input<Resolved>>>>()?;

        let output = self.output.resolve(parent_dir)?;

        Ok(Config {
            inputs,
            output,
            schema: self.schema,
            preset: self.preset,
            secret: self.secret.resolve(&reader_context),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::tests::TestEnvIO;
    use crate::core::valid::{ValidateInto, ValidationError, Validator};

    fn location<S: AsRef<str>>(s: S) -> Location<UnResolved> {
        Location(s.as_ref().to_string(), PhantomData)
    }

    fn to_headers(raw_headers: BTreeMap<String, TemplateString>) -> Headers<UnResolved> {
        Headers(Some(raw_headers), PhantomData)
    }

    #[test]
    fn test_headers_resolve() {
        let mut headers = BTreeMap::new();
        headers.insert("Authorization".to_owned(), "Bearer {{.env.TOKEN}}".into());

        let mut env_vars = HashMap::new();
        let token = "eyJhbGciOiJIUzI1NiIsInR5";
        env_vars.insert("TOKEN".to_owned(), token.to_owned());

        let unresolved_headers = to_headers(headers);

        let mut runtime = crate::core::runtime::test::init(None);
        runtime.env = Arc::new(TestEnvIO::init(env_vars));

        let reader_ctx = ConfigReaderContext {
            runtime: &runtime,
            vars: &Default::default(),
            headers: Default::default(),
        };

        let resolved_headers = unresolved_headers.resolve(&reader_ctx);

        let expected = TemplateString::from(format!("Bearer {token}").as_str());
        let actual = resolved_headers
            .as_btree_map()
            .as_ref()
            .unwrap()
            .get("Authorization")
            .unwrap()
            .to_owned();

        assert_eq!(
            actual, expected,
            "Authorization header should be resolved correctly"
        );
    }

    #[test]
    fn test_config_codec() {
        let mut headers = BTreeMap::new();
        headers.insert("user-agent".to_owned(), "tailcall-v1".into());
        let config = Config::default().inputs(vec![Input {
            source: Source::Curl {
                src: location("https://example.com"),
                headers: to_headers(headers),
                body: None,
                field_name: "test".to_string(),
                method: Some(Method::GET),
                is_mutation: None,
            },
        }]);
        let actual = serde_json::to_string_pretty(&config).unwrap();
        insta::assert_snapshot!(actual)
    }

    #[test]
    fn should_fail_when_invalid_merge_type_threshold() {
        let config_preset = PresetConfig {
            tree_shake: None,
            infer_type_names: None,
            merge_type: Some(2.0),
            consolidate_url: None,
            unwrap_single_field_types: None,
        };

        let transform_preset: Result<Preset, ValidationError<String>> =
            config_preset.validate_into().to_result();
        assert!(transform_preset.is_err());
    }

    #[test]
    fn should_use_user_provided_presets_when_provided() {
        let config_preset = PresetConfig {
            tree_shake: Some(true),
            infer_type_names: Some(true),
            merge_type: Some(0.5),
            consolidate_url: Some(1.0),
            unwrap_single_field_types: None,
        };
        let transform_preset: Preset = config_preset.validate_into().to_result().unwrap();
        let expected_preset = Preset::new()
            .infer_type_names(true)
            .tree_shake(true)
            .consolidate_url(1.0)
            .merge_type(0.5);
        assert_eq!(transform_preset, expected_preset);
    }

    #[test]
    fn test_location_resolve_with_url() {
        let json_source = r#""https://dummyjson.com/products""#;
        let de_source: Location<UnResolved> = serde_json::from_str(json_source).unwrap();
        let de_source = de_source.into_resolved(None);
        assert_eq!(de_source.0, "https://dummyjson.com/products");
        assert_eq!(de_source.1, PhantomData::<Resolved>);
    }

    #[test]
    fn test_is_empty() {
        let location_empty: Location<UnResolved> = serde_json::from_str(r#""""#).unwrap();
        let location_non_empty: Location<UnResolved> =
            serde_json::from_str(r#""https://dummyjson.com/products""#).unwrap();
        assert!(location_empty.is_empty());
        assert!(!location_non_empty.is_empty());
    }

    fn assert_deserialization_error(json: &str, expected_error: &str) {
        let config: Result<Config<UnResolved>, serde_json::Error> = serde_json::from_str(json);
        let actual = config.err().unwrap().to_string();
        assert_eq!(actual, expected_error);
    }

    #[test]
    fn test_raise_error_unknown_field_at_root_level() {
        let json = r#"{"input": "value"}"#;
        let expected_error =
            "unknown field `input`, expected one of `inputs`, `output`, `preset`, `schema`, `secret` at line 1 column 8";
        assert_deserialization_error(json, expected_error);
    }

    #[test]
    fn test_raise_error_unknown_field_in_inputs() {
        let json = r#"
            {"inputs": [{
                "curl": {
                    "src": "https://tailcall.run/graphql",
                    "headerss": {
                        "content-type": "application/json"
                    }
                }
            }]}
        "#;
        let expected_error =
            "unknown field `headerss`, expected one of `src`, `headers`, `method`, `body`, `isMutation`, `fieldName` at line 9 column 13";
        assert_deserialization_error(json, expected_error);

        let json = r#"
            {"inputs": [{
                "curls": {
                    "src": "https://tailcall.run/graphql",
                    "headerss": {
                        "content-type": "application/json"
                    }
                }
            }]}
        "#;
        let expected_error =
            "no variant of enum Source found in flattened data at line 9 column 13";
        assert_deserialization_error(json, expected_error);
    }

    #[test]
    fn test_raise_error_unknown_field_in_preset() {
        let json = r#"
            {"preset": {
                "mergeTypes": 1.0,
                "consolidateURL": 0.5
            }} 
        "#;
        let expected_error =
            "unknown field `mergeTypes`, expected one of `mergeType`, `consolidateURL`, `inferTypeNames`, `treeShake`, `unwrapSingleFieldTypes` at line 3 column 28";
        assert_deserialization_error(json, expected_error);
    }

    #[test]
    fn test_raise_error_unknown_field_in_output() {
        let json = r#"
          {"output": {
              "paths": "./output.graphql",
          }} 
        "#;
        let expected_error =
            "unknown field `paths`, expected `path` or `format` at line 3 column 21";
        assert_deserialization_error(json, expected_error);
    }

    #[test]
    fn test_raise_error_unknown_field_in_schema() {
        let json = r#"
          {"schema": {
              "querys": "Query",
          }} 
        "#;
        let expected_error =
            "unknown field `querys`, expected `query` or `mutation` at line 3 column 22";
        assert_deserialization_error(json, expected_error);
    }

    #[test]
    fn test_secret() {
        let mut env_vars = HashMap::new();
        let token = "eyJhbGciOiJIUzI1NiIsInR5";
        env_vars.insert("TAILCALL_SECRET".to_owned(), token.to_owned());

        let mut runtime = crate::core::runtime::test::init(None);
        runtime.env = Arc::new(TestEnvIO::init(env_vars));

        let reader_ctx = ConfigReaderContext {
            runtime: &runtime,
            vars: &Default::default(),
            headers: Default::default(),
        };

        let config =
            Config::default().secret(TemplateString::parse("{{.env.TAILCALL_SECRET}}").unwrap());
        let resolved_config = config.into_resolved("", reader_ctx).unwrap();

        let actual = resolved_config.secret;
        let expected = TemplateString::from("eyJhbGciOiJIUzI1NiIsInR5");

        assert_eq!(actual, expected);
    }
}
