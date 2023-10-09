use super::update_group_by::GroupByTransform;
use crate::blueprint::from_config::{to_args, to_type, validate_field_type_exist};
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::http::HttpTransform;
use crate::blueprint::transformers::update_const::UpdateConstTransform;
use crate::blueprint::transformers::update_inline::UpdateInlineTransform;
use crate::blueprint::transformers::update_modify::ModifyTransform;
use crate::blueprint::transformers::update_unsafe::UpdateUnsafeTransform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{
  Blueprint, Definition, FieldDefinition, InputFieldDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
  ObjectTypeDefinition,
};
use crate::config;
use crate::config::Config;
use crate::valid::{ValidExtensions, VectorExtension};

pub struct ObjectTransform {
  pub type_of: config::Type,
  pub name: String,
}

impl From<ObjectTransform> for Transform<Config, Blueprint, String> {
  fn from(value: ObjectTransform) -> Self {
    let name = value.name.clone();
    Transform::new(move |config, blueprint| value.transform(config, blueprint).trace(name.as_str()))
  }
}
impl ObjectTransform {
  fn transform(self, config: &Config, mut blueprint: Blueprint) -> Valid<Blueprint> {
    let definition = self
      .type_of
      .fields
      .iter()
      .validate_all(|(name, field)| {
        validate_field_type_exist(config, field)
          .validate_or(to_field(config, &self.type_of, name, field))
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
      if config.input_types().contains(&self.name) {
        to_input_object_type_definition(object_type_definition)?
      } else if self.type_of.interface {
        to_interface_type_definition(object_type_definition)?
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
) -> Valid<Option<FieldDefinition>> {
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

  let mut field_transformer = Transform::from(HttpTransform { field: field.clone() })
    + Transform::from(GroupByTransform { field: field.clone() })
    + Transform::from(UpdateUnsafeTransform { field: field.clone() })
    + Transform::from(UpdateConstTransform { field: field.clone() })
    + Transform::from(UpdateInlineTransform { field: field.clone(), type_info: type_of.clone() });

  let modify = field.modify.clone();

  if let Some(modify) = modify.as_ref() {
    if modify.omit {
      return Ok(None);
    }
  }
  field_transformer = field_transformer + Transform::from(ModifyTransform { modify, type_info: type_of.clone() });

  let field = field_transformer.transform(config, field_definition)?;
  Ok(Some(field))
}

fn to_input_object_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
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

fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
  Valid::Ok(Definition::InterfaceTypeDefinition(InterfaceTypeDefinition {
    name: definition.name,
    fields: definition.fields,
    description: definition.description,
  }))
}
