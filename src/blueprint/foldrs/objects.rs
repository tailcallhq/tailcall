use clap::builder::Str;

use crate::blueprint::foldrs::fd::update_const::ConstFold;
use crate::blueprint::foldrs::fd::update_group_by::GroupByFold;
use crate::blueprint::foldrs::fd::update_inline::InlineFold;
use crate::blueprint::foldrs::fd::update_modify::ModifyFold;
use crate::blueprint::foldrs::fd::update_unsafe::UnsafeFold;
use crate::blueprint::foldrs::http::HttpFold;
use crate::blueprint::from_config::{to_args, to_type, validate_field_type_exist};
use crate::blueprint::transform::Transform;
use crate::blueprint::{
  Blueprint, Definition, FieldDefinition, InputFieldDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
  ObjectTypeDefinition,
};
use crate::config;
use crate::config::{Config, ConstField, ModifyField};
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions, VectorExtension};

pub struct ObjectsFold {
  pub type_of: config::Type,
  pub name: String,
}

impl TryFolding for ObjectsFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    let definition = self
      .type_of
      .fields
      .iter()
      .validate_all(|(name, field)| {
        validate_field_type_exist(cfg, field)
          .validate_or(to_field(cfg, &self.type_of, name, field))
          .trace(name)
      })
      .map(|fields| {
        Definition::ObjectTypeDefinition(ObjectTypeDefinition {
          name: self.name.clone(),
          description: self.type_of.doc.clone(),
          fields: fields.into_iter().flatten().collect(),
          implements: self.type_of.implements.clone(),
        })
      })?;
    // TODO: remove the `clone` operation
    let definition = if let Definition::ObjectTypeDefinition(object_type_definition) = definition.clone() {
      if cfg.input_types().contains(&self.name) {
        to_input_object_type_definition(object_type_definition).trace(&self.name)?
      } else if self.type_of.interface {
        to_interface_type_definition(object_type_definition).trace(&self.name)?
      } else {
        definition
      }
    } else {
      definition
    };

    blueprint.definitions.push(definition);

    Ok(blueprint)
  }
}

#[allow(clippy::too_many_arguments)]
fn to_field(
  config: &Config,
  type_of: &config::Type,
  name: &str,
  field: &config::Field,
) -> Valid<Option<FieldDefinition>, String> {
  let directives = field.resolvable_directives();
  if directives.len() > 1 {
    return Valid::fail(format!("Multiple resolvers detected [{}]", directives.join(", ")));
  }

  let field_type = &field.type_of;
  let args = to_args(field)?;

  let field_definition = FieldDefinition {
    name: name.to_owned(),
    description: field.doc.clone(),
    args,
    of_type: to_type(field_type, field.list, field.required, field.list_type_required),
    directives: Vec::new(),
    resolver: None,
  };

  let mut field_fold = HttpFold { field: field.clone() }
    .and(GroupByFold { field: field.clone() })
    .and(UnsafeFold { field: field.clone() })
    .and(ConstFold { field: field.clone() })
    .and(InlineFold { field: field.clone(), type_info: type_of.clone() });

  let modify = field.modify.clone();

  if let Some(modify) = modify.as_ref() {
    if modify.omit {
      return Ok(None);
    }
  }
  field_fold = field_fold.and(ModifyFold { modify, type_info: type_of.clone() });

  let field = field_fold.transform(config, field_definition)?;
  Ok(Some(field))
}

fn to_input_object_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
  Valid::Ok(Definition::InputObjectTypeDefinition(InputObjectTypeDefinition {
    name: definition.name,
    fields: definition
      .fields
      .iter()
      .map(|field| InputFieldDefinition {
        name: field.name.clone(),
        description: field.description.clone(),
        default_value: None,
        of_type: field.of_type.clone(),
      })
      .collect(),
    description: definition.description,
  }))
}

fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
  Valid::Ok(Definition::InterfaceTypeDefinition(InterfaceTypeDefinition {
    name: definition.name,
    fields: definition.fields,
    description: definition.description,
  }))
}
