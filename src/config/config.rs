use std::collections::{BTreeMap, HashSet};

use async_graphql::parser::types::ServiceDocument;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::batch::Batch;
use crate::http::Method;
use crate::json::JsonSchema;
use crate::mustache::Mustache;
use crate::path::{path_deserialize, path_serialize, Path};
use anyhow::Result;

use super::{Proxy, Server};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Setters)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub server: Server,
    pub graphql: GraphQL,
}

impl Config {
    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn proxy(&self) -> Option<Proxy> {
        self.server.proxy.clone()
    }

    pub fn output_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();

        if let Some(ref query) = &self.graphql.schema.query {
            types.insert(query);
        }

        if let Some(ref mutation) = &self.graphql.schema.mutation {
            types.insert(mutation);
        }

        for (_, type_of) in self.graphql.types.iter() {
            if type_of.interface.unwrap_or(false) || !type_of.fields.is_empty() {
                for (_, field) in type_of.fields.iter() {
                    types.insert(&field.type_of);
                }
            }
        }
        types
    }

    pub fn input_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();
        for (_, type_of) in self.graphql.types.iter() {
            if !type_of.interface.unwrap_or(false) {
                for (_, field) in type_of.fields.iter() {
                    if let Some(ref args) = field.args {
                        for (_, arg) in args.iter() {
                            types.insert(&arg.type_of);
                        }
                    }
                }
            }
        }
        types
    }

    pub fn find_type(&self, name: String) -> Option<&Type> {
        self.graphql.types.get(&name)
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn into_service_document(&self) -> ServiceDocument {
        (self.clone()).into()
    }

    pub fn to_sdl(&self) -> String {
        let doc = self.into_service_document();
        crate::document::print(doc)
    }

    pub fn query(mut self, query: &str) -> Self {
        self.graphql.schema.query = Some(query.to_string());
        self
    }

    pub fn types(mut self, types: Vec<(&str, Type)>) -> Self {
        let mut graphql_types = BTreeMap::new();
        for (name, type_) in types {
            graphql_types.insert(name.to_string(), type_);
        }
        self.graphql.types = graphql_types;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Type {
    pub fields: BTreeMap<String, Field>,
    pub doc: Option<String>,
    pub interface: Option<bool>,
    pub implements: Option<Vec<String>>,
    #[serde(rename = "enum")]
    pub variants: Option<Vec<String>>,
    pub scalar: Option<bool>,
}
impl Type {
    pub fn is_interface(&self) -> bool {
        matches!(self.interface, Some(true))
    }

    pub fn fields(mut self, fields: Vec<(&str, Field)>) -> Self {
        let mut graphql_fields = BTreeMap::new();
        for (name, field) in fields {
            graphql_fields.insert(name.to_string(), field);
        }
        self.fields = graphql_fields;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GraphQL {
    pub schema: RootSchema,
    pub types: BTreeMap<String, Type>,
    pub unions: Option<Vec<Union>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Setters)]
#[setters(strip_option)]
pub struct RootSchema {
    pub query: Option<String>,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Setters)]
#[setters(strip_option)]
pub struct Field {
    pub type_of: String,
    pub list: Option<bool>,
    pub required: Option<bool>,
    pub list_type_required: Option<bool>,
    pub args: Option<BTreeMap<String, Arg>>,
    pub doc: Option<String>,
    pub modify: Option<ModifyField>,
    pub inline: Option<InlineType>,
    pub http: Option<Http>,
    #[serde(rename = "unsafe")]
    pub unsafe_operation: Option<Unsafe>,
    pub batch: Option<Batch>,
}

impl Field {
    pub fn has_resolver(&self) -> bool {
        self.http.is_some() || self.unsafe_operation.is_some()
    }
    pub fn has_batched_resolver(&self) -> bool {
        if let Some(http) = self.http.as_ref() {
            http.match_key.is_some() || http.match_path.is_some()
        } else {
            false
        }
    }
    pub fn to_list(mut self) -> Self {
        self.list = Some(true);
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Unsafe {
    pub script: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModifyField {
    pub name: Option<String>,
    pub omit: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InlineType {
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Arg {
    pub type_of: String,
    pub list: Option<bool>,
    pub required: Option<bool>,
    pub doc: Option<String>,
    pub modify: Option<ModifyField>,
    pub default_value: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Union {
    pub name: String,
    pub types: Vec<String>,
    pub doc: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Http {
    #[serde(deserialize_with = "path_deserialize", serialize_with = "path_serialize")]
    pub path: Path,
    pub method: Option<Method>,
    pub query: Option<BTreeMap<String, Mustache>>,
    pub input: Option<JsonSchema>,
    pub output: Option<JsonSchema>,
    pub body: Option<Mustache>,
    pub match_path: Option<Vec<String>>,
    pub match_key: Option<String>,
    #[serde(rename = "baseURL", serialize_with = "super::url::serialize")]
    pub base_url: Option<url::Url>,
    pub headers: Option<BTreeMap<String, String>>,
}

impl Http {
    pub fn batch_key(mut self, key: &str) -> Self {
        self.match_key = Some(key.to_string());
        self
    }
}

impl Config {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_sdl(sdl: &str) -> Result<Self> {
        let doc = async_graphql::parser::parse_schema(sdl)?;

        Ok(Config::from(doc))
    }

    pub fn n_plus_one(&self) -> Vec<Vec<(String, String)>> {
        super::n_plus_one::n_plus_one(self)
    }
}
