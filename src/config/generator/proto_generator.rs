#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;
use std::ops::Deref;

use anyhow::anyhow;
use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use strum_macros::Display;

use crate::config::generator::from_proto::prebuild_config;
use crate::config::{Config, Type};

#[derive(Default)]
pub struct ConfigWrapper {
    pub config: Config,
    pub types: BTreeMap<String, Vec<String>>,
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
    format_enum: Box<dyn Fn(Vec<String>) -> Vec<String>>,
    format_ty: Box<dyn Fn(Vec<String>) -> Vec<String>>,
}

impl ProtoGeneratorFxn {
    pub fn new(
        is_mutation: Box<dyn Fn(&str) -> bool>,
        format_enum: Box<dyn Fn(Vec<String>) -> Vec<String>>,
        format_ty: Box<dyn Fn(Vec<String>) -> Vec<String>>,
    ) -> Self {
        Self { is_mutation, format_enum, format_ty }
    }
}

impl Default for ProtoGeneratorFxn {
    fn default() -> Self {
        let fmt = |x: Vec<String>| x;
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
        let mut pre_built_wrapper = prebuild_config(
            descriptor_sets,
            &self.generator_config,
            self.generator_config.option,
        )?;

        if self.generator_config.option == Options::AppendPkgId {
            let original_enums = pre_built_wrapper
                .types
                .get(&DescriptorType::Enum.to_string())
                .unwrap()
                .clone();
            let original_messages = pre_built_wrapper
                .types
                .get(&DescriptorType::Message.to_string())
                .unwrap()
                .clone();

            let updated_enums = (self.generator_config.generator_fxn.format_enum)(
                pre_built_wrapper
                    .types
                    .get(&DescriptorType::Enum.to_string())
                    .unwrap()
                    .clone(),
            );
            let updated_messages = (self.generator_config.generator_fxn.format_ty)(
                pre_built_wrapper
                    .types
                    .get(&DescriptorType::Message.to_string())
                    .unwrap()
                    .clone(),
            );
            if original_enums.len() != updated_enums.len()
                || original_messages.len() != updated_messages.len()
            {
                return Err(anyhow!("Invalid length of enums or messages expected:\nEnums: {} got {}\nMessages: {} got {}", original_enums.len(), updated_enums.len(), original_messages.len(), updated_messages.len()));
            }
            let mut updated_enums_map = BTreeMap::new();
            let mut updated_messages_map = BTreeMap::new();

            original_messages
                .into_iter()
                .zip(updated_messages)
                .for_each(|(k, v)| {
                    updated_messages_map.insert(k, v);
                });

            original_enums
                .into_iter()
                .zip(updated_enums)
                .for_each(|(k, v)| {
                    updated_enums_map.insert(k, v);
                });

            update_stuff(&mut pre_built_wrapper.config, updated_enums_map);
            update_stuff(&mut pre_built_wrapper.config, updated_messages_map);
        }

        Ok(pre_built_wrapper.config)
    }
}

fn update_stuff(cfg: &mut Config, updated_stuff: BTreeMap<String, String>) {
    let mut new_types = BTreeMap::new();
    for (k, v) in cfg.types.iter_mut() {
        let k = if let Some(new_enum) = updated_stuff.get(k) {
            new_enum.clone()
        } else {
            k.clone()
        };

        for (_, field) in v.fields.iter_mut() {
            if let Some(new_stuff) = updated_stuff.get(&field.type_of) {
                field.type_of = new_stuff.clone();
            }

            for (_, arg) in field.args.iter_mut() {
                if let Some(new_stuff) = updated_stuff.get(&arg.type_of) {
                    arg.type_of = new_stuff.clone();
                }
            }
        }
        new_types.insert(k, v.clone());
    }
    cfg.types = new_types;
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use convert_case::{Case, Casing};
    use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::config::generator::proto_generator::{
        ProtoGenerator, ProtoGeneratorConfig, ProtoGeneratorFxn,
    };

    fn get_proto_file_descriptor(name: &str) -> anyhow::Result<FileDescriptorProto> {
        let mut proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        proto_path.push("src");
        proto_path.push("grpc");
        proto_path.push("tests");
        proto_path.push("proto");
        proto_path.push(name);
        Ok(protox_parse::parse(
            name,
            std::fs::read_to_string(proto_path)?.as_str(),
        )?)
    }

    fn get_generator() -> ProtoGenerator {
        let is_mut = |x: &str| !x.starts_with("Get");
        let fmt = |x: Vec<String>| {
            x.into_iter()
                .map(|v| v.to_case(Case::Snake).to_lowercase())
                .collect()
        };
        ProtoGenerator::new(ProtoGeneratorConfig::new(
            Some("Query".to_string()),
            Some("Mutation".to_string()),
            ProtoGeneratorFxn::new(Box::new(is_mut), Box::new(fmt), Box::new(fmt)),
        ))
    }

    #[test]
    fn foo() -> anyhow::Result<()> {
        let gen = get_generator();

        let mut set = FileDescriptorSet::default();

        let news = get_proto_file_descriptor("news_enum.proto")?;
        let greetings = get_proto_file_descriptor("greetings.proto")?;
        let greetings_dup_methods = get_proto_file_descriptor("greetings_dup_methods.proto")?;

        set.file.push(news.clone());
        set.file.push(greetings.clone());
        set.file.push(greetings_dup_methods.clone());

        let config = gen.generate(vec![set])?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
