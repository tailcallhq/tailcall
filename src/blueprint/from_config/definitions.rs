#[allow(unused_imports)]
use async_graphql::InputType;

use crate::blueprint::*;
use crate::config::Union;
use crate::valid::Valid;

pub fn to_scalar_type_definition(name: &str) -> Valid<Definition, String> {
  Valid::succeed(Definition::ScalarTypeDefinition(ScalarTypeDefinition {
    name: name.to_string(),
    directive: Vec::new(),
    description: None,
  }))
}

pub fn to_union_type_definition((name, u): (&String, &Union)) -> Definition {
  Definition::UnionTypeDefinition(UnionTypeDefinition {
    name: name.to_owned(),
    description: u.doc.clone(),
    directives: Vec::new(),
    types: u.types.clone(),
  })
}

pub fn to_input_object_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
  Valid::succeed(Definition::InputObjectTypeDefinition(InputObjectTypeDefinition {
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

pub fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
  Valid::succeed(Definition::InterfaceTypeDefinition(InterfaceTypeDefinition {
    name: definition.name,
    fields: definition.fields,
    description: definition.description,
  }))
}
