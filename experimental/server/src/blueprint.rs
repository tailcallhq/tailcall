// TODO: Can we somehow share this code with the Scala CLI implementation?

use serde::Deserialize;
use std::collections::HashMap;

use crate::digest::Digest;

// TODO: This should deserialize to a function, as with zio-compose
#[derive(Deserialize)]
struct Lambda(usize);

#[derive(Deserialize)]
struct DynamicValue(usize);

#[derive(Deserialize)]
pub enum TypeTag {
    NamedType(NamedType),
    ListType(ListType),
}

#[derive(Deserialize)]
pub enum BlueprintDefinition {
    ObjectTypeDefinition(ObjectTypeDefinition),
    InputObjectTypeDefinition(InputFieldDefinition),
    SchemaDefinition(SchemaDefinition),
    InputFieldDefinition(InputFieldDefinition),
    FieldDefinition(FieldDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    Directive(Directive),
}

pub struct Blueprint {
    pub digest: Digest,
    pub definitions: Vec<BlueprintDefinition>,
}

impl Blueprint {
    pub fn new(digest: Digest, definitions: Vec<BlueprintDefinition>) -> Self {
        Blueprint {
            digest,
            definitions,
        }
    }
}

#[derive(Deserialize)]
pub struct ObjectTypeDefinition {
    name: String,
    fields: Vec<FieldDefinition>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct InputObjectTypeDefinition {
    name: String,
    fields: Vec<InputFieldDefinition>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct SchemaDefinition {
    query: Option<String>,
    mutation: Option<String>,
    subscription: Option<String>,
    directives: Vec<Directive>,
}

#[derive(Deserialize)]
pub struct InputFieldDefinition {
    name: String,
    ofType: TypeTag,
    defaultValue: Option<DynamicValue>,
}

#[derive(Deserialize)]
pub struct FieldDefinition {
    name: String,
    args: Vec<InputFieldDefinition>,
    ofType: TypeTag,
    resolver: Option<Lambda>,
    directives: Vec<Directive>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct ScalarTypeDefinition {
    name: String,
    directive: Vec<Directive>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct Directive {
    name: String,
    arguments: HashMap<String, DynamicValue>,
    index: usize,
}

#[derive(Deserialize)]
pub struct NamedType {
    name: String,

    #[serde(rename = "nonNull")]
    isNonNull: bool,
}

#[derive(Deserialize)]
pub struct ListType {
    ofType: Box<TypeTag>,

    #[serde(rename = "nonNull")]
    isNonNull: bool,
}
