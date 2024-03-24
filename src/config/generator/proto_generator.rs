use std::collections::BTreeMap;
use std::ops::Deref;
use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use strum_macros::Display;

use crate::config::{Config, Type};
use crate::config::generator::from_proto::prebuild_config;


#[derive(Default)]
pub struct ConfigWrapper {
    pub config: Config,
    pub types: BTreeMap<String, Vec<String>>
}

impl ConfigWrapper {
    pub fn insert_ty(&mut self, key: String, val: Type, ty: String) {
        if let Some(val) = self.types.get_mut(&ty) {
            val.push(key.clone());
        } else {
            self.types.insert(ty, vec![key.clone()]);
        }

        self.config.types.insert(key, val);
    }
    pub fn get_ty(&self, key: &str) -> Type {
        self.config.types.get(key).cloned().unwrap_or_default()
    }
}

impl Deref for ConfigWrapper {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[derive(Display, Clone, Copy)]
pub enum DescriptorType {
    Enum,
    Message,
    Method,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Options {
    // TODO rename
    AppendPkgId,
    FailIfCollide,
    Merge,
}

pub struct ProtoGeneratorFxn {
    is_mutation: Box<dyn Fn(&str) -> bool>,
    format_enum: Box<dyn Fn(Vec<String>) -> BTreeMap<String, String>>,
    format_ty: Box<dyn Fn(Vec<String>) -> BTreeMap<String,String>>,
}

impl ProtoGeneratorFxn {
    pub fn new(
        is_mutation: Box<dyn Fn(&str) -> bool>,
        format_enum: Box<dyn Fn(Vec<String>) ->  BTreeMap<String,String>>,
        format_ty: Box<dyn Fn(Vec<String>) ->  BTreeMap<String,String>>,
    ) -> Self {
        Self {
            is_mutation,
            format_enum,
            format_ty,
        }
    }
}

impl Default for ProtoGeneratorFxn {
    fn default() -> Self {
        let fmt = |x: Vec<String>| {
            let mut map = BTreeMap::new();
            x.into_iter().for_each(|v| { map.insert(v.clone(), v); });
            map
        };
        Self {
            is_mutation: Box::new(|_| false),
            format_enum: Box::new(fmt),
            format_ty: Box::new(fmt),
        }
    }
}

#[derive(Setters)]
pub struct ProtoGeneratorConfig {
    query: String,
    mutation: String,
    generator_fxn: ProtoGeneratorFxn,
    option: Options,
}

impl ProtoGeneratorConfig {
    pub fn new(
        query: Option<String>,
        mutation: Option<String>,
        generator_fxn: ProtoGeneratorFxn,
    ) -> Self {
        Self {
            query: query.unwrap_or_else(|| "Query".to_string()),
            mutation: mutation.unwrap_or_else(|| "Mutation".to_string()),
            generator_fxn,
            option: Options::AppendPkgId,
        }
    }

    pub fn is_mutation(&self, name: &str) -> bool {
        (self.generator_fxn.is_mutation)(name)
    }
    pub fn get_query(&self) -> &str {
        self.query.as_str()
    }

    pub fn get_mutation(&self) -> &str {
        self.mutation.as_str()
    }
}

impl Default for ProtoGeneratorConfig {
    fn default() -> Self {
        Self {
            query: "Query".to_string(),
            mutation: "Mutation".to_string(),
            generator_fxn: ProtoGeneratorFxn::default(),
            option: Options::AppendPkgId,
        }
    }
}
struct ProtoGenerator {
    generator_config: ProtoGeneratorConfig,
}

impl ProtoGenerator {
    pub fn new(generator_config: ProtoGeneratorConfig) -> Self {
        Self { generator_config }
    }

    pub fn generate(&self, descriptor_sets: Vec<FileDescriptorSet>) -> anyhow::Result<Config> {
        let mut config = Config::default();
        let pre_built_wrapper = prebuild_config(descriptor_sets, &self.generator_config, self.generator_config.option)?;
        config.schema = pre_built_wrapper.config.schema;

        match self.generator_config.option {
            Options::AppendPkgId => {

            }
            _ => (),
        }

        Ok(config)
    }
}