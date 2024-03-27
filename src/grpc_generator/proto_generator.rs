#![allow(dead_code)] // TODO check what to do..

use std::collections::BTreeMap;
use std::ops::Deref;

use anyhow::anyhow;
use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use strum_macros::Display;

use crate::config::{Config, Type};
use crate::grpc_generator::from_proto::prebuild_config;

pub(super) static DEFAULT_SPECTATOR: &str = "_";
pub(super) static FIELD_TY: &str = "field_ty_unique_id";
pub(super) static ARG_TY: &str = "arg_ty_unique_id";

/// Enum to represent the type of the descriptor
#[derive(Display, Clone)]
pub enum DescriptorType {
    Enum,
    Message,
    Query(String),
    Mutation(String),
}

/// Options to be used while generating the proto
/// Interactive: Allows manual handling of collisions
/// FailIfCollide: Fails if there is a collision
/// Merge: Merges the types in case of collision
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Options {
    // TODO rename
    Interactive,
    FailIfCollide,
    Merge,
}

/// ProtoGenerator to generate the config from the proto files
pub struct ProtoGenerator {
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

        if self.generator_config.option == Options::Interactive {
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
                let original_qry = pre_built_wrapper.types.get(qry).unwrap().clone();

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
                let original_mutation = pre_built_wrapper.types.get(mutation).unwrap().clone();
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

/// Contains the configuration for the config generator
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
            option: Options::Interactive,
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
            option: Options::Interactive,
        }
    }
}

/// Contains the functions to be used for interactively generating the config
#[derive(Setters)]
pub struct ProtoGeneratorFxn {
    pub is_mutation: Box<dyn Fn(&str) -> bool>,
    pub format_enum: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_ty: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_query: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    pub format_mutation: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
}

impl ProtoGeneratorFxn {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        is_mutation: Box<dyn Fn(&str) -> bool>,
        format_enum: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_ty: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_query: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
        format_mutation: Box<dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>>,
    ) -> Self {
        Self {
            is_mutation,
            format_enum,
            format_ty,
            format_query,
            format_mutation,
        }
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

/// Contains package id and name in case of message or enum
/// And name of the field with optional args in case of query or mutation
#[derive(Clone)]
pub struct FieldHolder {
    descriptor_type: DescriptorType,
    package_id: String,
    name: String,
    updated_name: Option<String>,
    args: Vec<ArgsHolder>,
}

/// Contains name of the field and updated name of the field
#[derive(Clone)]
pub struct ArgsHolder {
    name: String,
    updated_name: Option<String>,
}

impl FieldHolder {
    /// returns the original name in proto file
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    /// returns the package id of the respective proto file
    pub fn get_package_id(&self) -> String {
        self.package_id.clone()
    }
    /// inserts new name for the field
    pub fn insert_updated_name(&mut self, updated_name: String) {
        self.updated_name = Some(updated_name);
    }
    /// used to get the default name of the field
    /// used to store original name of field,
    /// or in case there is no updated name
    pub fn get_default_name(&self) -> String {
        let pkg_id = self.get_package_id();
        let name = self.get_name();
        if pkg_id.eq(FIELD_TY) || pkg_id.eq(ARG_TY) {
            name
        } else {
            format!("{}{}{}", name, DEFAULT_SPECTATOR, pkg_id)
        }
    }

    /// returns updated name if present else returns default name
    pub fn get_updated_name(&self) -> String {
        self.updated_name
            .clone()
            .unwrap_or_else(|| self.get_default_name())
    }
}

impl ArgsHolder {
    /// returns the original name or arg in proto file
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// returns updated name if present else returns original name
    pub fn get_updated_name(&self) -> String {
        self.updated_name.clone().unwrap_or_else(|| self.get_name())
    }

    /// inserts new name for the arg
    pub fn insert_updated_name(&mut self, name: String) {
        self.updated_name = Some(name);
    }
}

// internal

/// Wrapper used for pre-building config to flatten the types
/// and store the types in a map
#[derive(Default)]
pub(super) struct ConfigWrapper {
    pub(super) config: Config,
    pub(super) types: BTreeMap<String, Vec<FieldHolder>>,
}

impl ConfigWrapper {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn insert_ty(
        &mut self,
        key: String,
        val: Type,
        ty: String,
        descriptor_type: DescriptorType,
    ) {
        let split = key.rsplitn(2, DEFAULT_SPECTATOR).collect::<Vec<&str>>();
        if let [name, pkg_id] = &split[..] {
            // is message or enum
            if let Some(val) = self.types.get_mut(&ty) {
                val.push(FieldHolder {
                    descriptor_type,
                    package_id: pkg_id.to_string(),
                    name: name.to_string(),
                    updated_name: None,
                    args: vec![],
                });
            } else {
                self.types.insert(
                    ty,
                    vec![FieldHolder {
                        descriptor_type,
                        package_id: pkg_id.to_string(),
                        name: name.to_string(),
                        updated_name: None,
                        args: vec![],
                    }],
                );
            }
        } else {
            // is query or mutation
            if let Some(vec) = self.types.get_mut(&ty) {
                for (k, field) in &val.fields {
                    let mut args = vec![];
                    for arg_k in field.args.keys() {
                        args.push(ArgsHolder { name: arg_k.clone(), updated_name: None });
                    }
                    vec.push(FieldHolder {
                        descriptor_type: descriptor_type.clone(),
                        package_id: FIELD_TY.to_string(),
                        name: k.clone(),
                        updated_name: None,
                        args,
                    });
                }
            } else {
                let mut vec = vec![];
                for (k, field) in &val.fields {
                    let mut args = vec![];
                    for arg_k in field.args.keys() {
                        args.push(ArgsHolder { name: arg_k.clone(), updated_name: None });
                    }
                    vec.push(FieldHolder {
                        descriptor_type: descriptor_type.clone(),
                        package_id: FIELD_TY.to_string(),
                        name: k.clone(),
                        updated_name: None,
                        args,
                    });
                }
                self.types.insert(ty, vec);
            }
        }

        self.config.types.insert(key, val);
    }
    pub(super) fn get_ty(&self, key: &str) -> Type {
        self.config.types.get(key).cloned().unwrap_or_default()
    }
}

impl Deref for ConfigWrapper {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

struct UpdateMapHolder {
    ty_field: BTreeMap<String, String>,
    args: BTreeMap<String, String>,
}

fn get_updated_map(
    original: Vec<FieldHolder>,
    func: &dyn Fn(Vec<FieldHolder>) -> Vec<FieldHolder>,
    ty: String,
) -> anyhow::Result<UpdateMapHolder> {
    let original_len = original.len();
    let mut updated_fields_map = BTreeMap::new();
    let mut updated_args_map = BTreeMap::new();
    let updated = func(original);
    if original_len != updated.len() {
        return Err(anyhow!(
            "Invalid length of {} expected length: {} found {}",
            ty,
            original_len,
            updated.len()
        ));
    }
    updated.into_iter().for_each(|v| {
        updated_fields_map.insert(v.get_default_name(), v.get_updated_name());
        v.args.into_iter().for_each(|arg| {
            updated_args_map.insert(arg.get_name(), arg.get_updated_name());
        });
    });
    Ok(UpdateMapHolder { ty_field: updated_fields_map, args: updated_args_map })
}

fn update_qry_mut(cfg: &mut Config, updated_stuff: UpdateMapHolder, ty: &String) {
    let update_ty_fields = updated_stuff.ty_field;
    let update_args = updated_stuff.args;

    if let Some(v) = cfg.types.get_mut(ty) {
        let mut fields = BTreeMap::new();
        for (k, v) in v.fields.iter_mut() {
            let mut args = BTreeMap::new();
            for (arg_k, v) in v.args.iter() {
                let arg_k = if let Some(arg_k) = update_args.get(arg_k) {
                    arg_k.clone()
                } else {
                    arg_k.clone()
                };
                args.insert(arg_k, v.clone());
            }
            v.args = args;
            let k = if let Some(k) = update_ty_fields.get(k) {
                k.clone()
            } else {
                k.clone()
            };
            fields.insert(k, v.clone());
        }
        v.fields = fields;
    }
}

fn update_type_fields(cfg: &mut Config, updated_stuff: UpdateMapHolder) {
    let updated_ty_fields = updated_stuff.ty_field;
    let mut new_types = BTreeMap::new();
    for (k, v) in cfg.types.iter_mut() {
        let k = if let Some(new_enum) = updated_ty_fields.get(k) {
            new_enum.clone()
        } else {
            k.clone()
        };

        for (_, field) in v.fields.iter_mut() {
            if let Some(new_stuff) = updated_ty_fields.get(&field.type_of) {
                field.type_of = new_stuff.clone();
            }

            for (_, arg) in field.args.iter_mut() {
                if let Some(new_stuff) = updated_ty_fields.get(&arg.type_of) {
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

    use crate::grpc_generator::proto_generator::{
        DescriptorType, FieldHolder, ProtoGenerator, ProtoGeneratorConfig, ProtoGeneratorFxn,
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
            x.iter_mut().for_each(|v| {
                // update query/mutation
                match v.descriptor_type {
                    DescriptorType::Query(_) => {
                        let updated_name = v
                            .get_default_name()
                            .to_case(Case::Alternating)
                            .to_case(Case::Camel);
                        v.insert_updated_name(format!("{}_myqry", updated_name));
                        v.args.iter_mut().for_each(|arg| {
                            let updated_name =
                                arg.get_name().to_case(Case::Lower).to_case(Case::Camel);
                            arg.insert_updated_name(updated_name);
                        });
                    }
                    DescriptorType::Mutation(_) => {
                        let updated_name = v.get_default_name().to_case(Case::Kebab);
                        v.insert_updated_name(format!("{}_mymut", updated_name));
                        v.args.iter_mut().for_each(|arg| {
                            let updated_name =
                                arg.get_name().to_case(Case::Upper).to_case(Case::Camel);
                            arg.insert_updated_name(updated_name);
                        });
                    }
                    _ => (),
                }
            });
            x
        };

        ProtoGenerator::new(ProtoGeneratorConfig::new(
            Some("Query".to_string()),
            Some("Mutation".to_string()),
            ProtoGeneratorFxn::new(
                Box::new(is_mut),
                Box::new(fmt),
                Box::new(fmt),
                Box::new(fmt_qey_mut),
                Box::new(fmt_qey_mut),
            ),
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
