#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;
use std::ops::Deref;

use anyhow::anyhow;
use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use strum_macros::Display;

use crate::config::generator::from_proto::prebuild_config;
use crate::config::{Config, Type};

#[derive(Clone)]
pub struct FieldHolder {
    package_id: String,
    name: String,
    updated_name: Option<String>,
}

impl FieldHolder {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_package_id(&self) -> String {
        self.package_id.clone()
    }
    pub fn insert_updated_name(&mut self, updated_name: String) {
        self.updated_name = Some(updated_name);
    }
    pub fn get_default_name(&self) -> String {
        let pkg_id = self.get_package_id();
        let name = self.get_name();
        if pkg_id.is_empty() {
            name
        }else {
            format!("{}{}{}", name, DEFAULT_SPECTATOR, pkg_id)
        }
    }
    pub fn get_updated_name(&self) -> String {
        self.updated_name
            .clone()
            .unwrap_or_else(|| self.get_default_name())
    }
}

pub(super) static DEFAULT_SPECTATOR: &str = "_";

#[derive(Default)]
pub struct ConfigWrapper {
    pub config: Config,
    pub types: BTreeMap<String, Vec<FieldHolder>>,
}

impl ConfigWrapper {
    pub fn insert_ty(&mut self, key: String, val: Type, ty: String) {
        let split = key.rsplitn(2, DEFAULT_SPECTATOR).collect::<Vec<&str>>();
        if let [name, pkg_id] = &split[..] {
            if let Some(val) = self.types.get_mut(&ty) {
                val.push(FieldHolder {
                    package_id: pkg_id.to_string(),
                    name: name.to_string(),
                    updated_name: None,
                });
            } else {
                self.types.insert(
                    ty,
                    vec![FieldHolder {
                        package_id: pkg_id.to_string(),
                        name: name.to_string(),
                        updated_name: None,
                    }],
                );
            }
        } else {
            self.types.insert(
                ty,
                vec![FieldHolder {
                    package_id: String::new(),
                    name: key.clone(),
                    updated_name: None,
                }],
            );
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
    pub is_mutation: Box<dyn Fn(&str) -> bool>,
    pub format_enum: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_ty: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_query: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_mutation: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
}

impl ProtoGeneratorFxn {
    pub fn new(
        is_mutation: Box<dyn Fn(&str) -> bool>,
        format_enum: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_ty: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_query: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_mutation: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    ) -> Self {
        Self { is_mutation, format_enum, format_ty, format_query, format_mutation }
    }
}

impl Default for ProtoGeneratorFxn {
    fn default() -> Self {
        let fmt = |x: Vec<FieldHolder>| x;
        Self {
            is_mutation: Box::new(|_| false),
            format_enum: Box::new(fmt),
            format_ty: Box::new(fmt),
            format_query: Box::new(fmt),
            format_mutation: Box::new(fmt),
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

            let updated_enums_map = get_updated_map(
                original_enums,
                &self.generator_config.generator_fxn.format_enum,
                DescriptorType::Enum.to_string(),
            )?;

            let updated_messages_map = get_updated_map(
                original_messages,
                &self.generator_config.generator_fxn.format_ty,
                DescriptorType::Message.to_string(),
            )?;

            if let Some(qry) = &pre_built_wrapper.schema.query {
                let original_qry = pre_built_wrapper
                    .types
                    .get(qry)
                    .unwrap()
                    .clone();

                let updated_qry_map = get_updated_map(
                    original_qry,
                    &self.generator_config.generator_fxn.format_query,
                    qry.clone(),
                )?;
                let mut cfg_clone = pre_built_wrapper.config.clone();
                update_qry_mut(&mut cfg_clone, updated_qry_map, qry);
                pre_built_wrapper.config = cfg_clone;
            }

            if let Some(mutation) = &pre_built_wrapper.schema.mutation {
                let original_mutation = pre_built_wrapper
                    .types
                    .get(mutation)
                    .unwrap()
                    .clone();
                let updated_mutation_map = get_updated_map(
                    original_mutation,
                    &self.generator_config.generator_fxn.format_mutation,
                    mutation.clone(),
                )?;
                let mut cfg_clone = pre_built_wrapper.config.clone();
                update_qry_mut(&mut cfg_clone, updated_mutation_map, mutation);
                pre_built_wrapper.config = cfg_clone;
            }

            update_type_fields(&mut pre_built_wrapper.config, updated_enums_map);
            update_type_fields(&mut pre_built_wrapper.config, updated_messages_map);
        }

        Ok(pre_built_wrapper.config)
    }
}

fn get_updated_map(
    original: Vec<FieldHolder>,
    func: &Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    ty: String,
) -> anyhow::Result<BTreeMap<String, String>> {
    let original_len = original.len();
    let mut map = BTreeMap::new();
    let updated = func(original);
    if original_len != updated.len() {
        return Err(anyhow!("Invalid length of {} expected length: {} found {}",ty, original_len, updated.len()));
    }
    updated.into_iter().for_each(|v| {
        map.insert(v.get_default_name(), v.get_updated_name());
    });
    Ok(map)
}

fn update_qry_mut(cfg: &mut Config, updated_stuff: BTreeMap<String, String>, ty: &String) {
    if let Some(v) = cfg.types.get_mut(ty) {
        let mut fields = BTreeMap::new();
        for (k,v) in v.fields.iter() {
            let k = if let Some(k) = updated_stuff.get(k) {
                k.clone()
            }else {
                k.clone()
            };
            fields.insert(k, v.clone());
        }
        v.fields = fields;
    }
}

fn update_type_fields(cfg: &mut Config, updated_stuff: BTreeMap<String, String>) {
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
        FieldHolder, ProtoGenerator, ProtoGeneratorConfig, ProtoGeneratorFxn,
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
        let fmt = |x: Vec<FieldHolder>| {
            x.into_iter()
                .map(|mut v| {
                    let updated_name = v.get_default_name().to_case(Case::Snake);
                    v.insert_updated_name(updated_name);
                    v
                })
                .collect()
        };
        let fmt_qey_mut = |mut x: Vec<FieldHolder>| {
            x.iter_mut()
                .for_each(|v| {
                    let updated_name = v.get_default_name().to_case(Case::Snake);
                    v.insert_updated_name(format!("{}_myqry_or_mut", updated_name));
                });
            x
        };

        ProtoGenerator::new(ProtoGeneratorConfig::new(
            Some("Query".to_string()),
            Some("Mutation".to_string()),
            ProtoGeneratorFxn::new(Box::new(is_mut), Box::new(fmt), Box::new(fmt), Box::new(fmt_qey_mut), Box::new(fmt_qey_mut)),
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
