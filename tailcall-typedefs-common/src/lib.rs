use std::collections::{BTreeMap, HashSet};

use async_graphql_parser::types::*;
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use schemars::schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

#[derive(Clone)]
pub struct Attrs {
    pub name: &'static str,
    pub repeatable: bool,
    pub locations: Vec<&'static str>,
}

pub fn to_directive_location(str: &str) -> DirectiveLocation {
    match str {
        "Schema" => DirectiveLocation::Schema,
        "Object" => DirectiveLocation::Object,
        "FieldDefinition" => DirectiveLocation::FieldDefinition,
        "EnumValue" => DirectiveLocation::EnumValue,
        _ => DirectiveLocation::FieldDefinition,
    }
}

pub fn from_directive_location(str: DirectiveLocation) -> String {
    match str {
        DirectiveLocation::Schema => String::from("SCHEMA"),
        DirectiveLocation::Object => String::from("OBJECT"),
        DirectiveLocation::FieldDefinition => String::from("FIELD_DEFINITION"),
        DirectiveLocation::EnumValue => String::from("ENUM_VALUE_DEFINITION"),
        _ => String::from("FIELD_DEFINITION"),
    }
}

fn get_description<'a>(schema: &'a SchemaObject) -> Option<&'a String> {
    schema
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref())
}

fn get_enum_values(obj: &Schema) -> Option<Vec<String>> {
    match obj {
        Schema::Object(schema_object) => {
            if let Some(enum_values) = &schema_object.enum_values {
                return Some(
                    enum_values
                        .iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<String>>(),
                );
            }
            None
        }
        _ => None,
    }
}

fn generate_enum_definition(enum_values: Option<Vec<String>>, name: &str) -> TypeSystemDefinition {
    let mut enum_values_defintions = vec![];
    if let Some(enum_values) = enum_values {
        for enum_value in enum_values {
            let formated_value: String = enum_value
                .to_string()
                .chars()
                .filter(|ch| ch != &'"')
                .collect();
            enum_values_defintions.push(pos(EnumValueDefinition {
                value: pos(Name::new(formated_value)),
                description: None,
                directives: vec![],
            }));
        }
    }

    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::Enum(EnumType { values: enum_values_defintions }),
        description: None,
        directives: vec![],
        extend: false,
    }))
}

fn generate_input_definition(schema: SchemaObject, name: &str) -> TypeSystemDefinition {
    let description = get_description(&schema);

    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::InputObject(InputObjectType {
            fields: generate_fields_definition(&schema),
        }),
        description: description.map(|inner| pos(inner.clone())),
        directives: vec![],
        extend: false,
    }))
}

fn generate_fields_definition(schema: &SchemaObject) -> Vec<Positioned<InputValueDefinition>> {
    let mut fields_definition = vec![];
    if let Some(sub_schemas) = schema.subschemas.clone() {
        if let Some(one_of) = sub_schemas.one_of {
            for schema in one_of {
                let schema_object = schema.into_object();
                fields_definition.extend(do_stuff(&schema_object));
            }

            return fields_definition;
        }
    }

    do_stuff(schema)
}

fn do_stuff(schema: &SchemaObject) -> Vec<Positioned<InputValueDefinition>> {
    let mut arguments = vec![];
    if let Some(properties) = schema
        .object
        .as_ref()
        .map(|object| object.properties.clone())
    {
        for (name, property) in properties.into_iter() {
            let property = property.into_object();
            let description = get_description(&property);
            let definition = pos(InputValueDefinition {
                description: description.map(|inner| pos(inner.to_owned())),
                name: pos(Name::new(name.to_owned())),
                ty: pos(get_field_type(name, property.clone())),
                default_value: None,
                directives: Vec::new(),
            });

            arguments.push(definition);
        }
    }

    arguments
}

fn get_field_type(name: String, schema: SchemaObject) -> Type {
    match schema.instance_type {
        Some(SingleOrVec::Single(typ))
            if matches!(
                *typ,
                InstanceType::Null
                    | InstanceType::Boolean
                    | InstanceType::Number
                    | InstanceType::String
                    | InstanceType::Integer
            ) =>
        {
            Type {
                nullable: false,
                base: BaseType::Named(Name::new(get_instace_type(&*typ))),
            }
        }
        Some(SingleOrVec::Vec(typ))
            if matches!(
                typ.first().unwrap(),
                InstanceType::Null
                    | InstanceType::Boolean
                    | InstanceType::Number
                    | InstanceType::String
                    | InstanceType::Integer
            ) =>
        {
            Type {
                nullable: true,
                base: BaseType::Named(Name::new(get_instace_type(typ.first().unwrap()))),
            }
        }
        _ => {
            if let Some(arr_valid) = schema.array {
                if let Some(SingleOrVec::Single(schema)) = arr_valid.items {
                    return Type {
                        nullable: true,
                        base: BaseType::List(Box::new(get_field_type(name, schema.into_object()))),
                    };
                } else if let Some(SingleOrVec::Vec(schemas)) = arr_valid.items {
                    return Type {
                        nullable: true,
                        base: BaseType::List(Box::new(get_field_type(
                            name,
                            schemas[0].clone().into_object(),
                        ))),
                    };
                } else {
                    return Type { nullable: true, base: BaseType::Named(Name::new("JSON")) };
                }
            } else if let Some(typ) = schema.object.clone() {
                if !typ.properties.is_empty() {
                    Type { nullable: true, base: BaseType::Named(Name::new(name)) }
                } else {
                    Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
                }
            } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
                let list = if let Some(list) = sub_schema.any_of {
                    list
                } else if let Some(list) = sub_schema.all_of {
                    list
                } else if let Some(list) = sub_schema.one_of {
                    list
                } else {
                    return Type { nullable: true, base: BaseType::Named(Name::new("JSON")) };
                };
                let first = list.first().unwrap();
                match first {
                    Schema::Object(obj) => {
                        let nm = obj
                            .reference
                            .to_owned()
                            .unwrap()
                            .split('/')
                            .last()
                            .unwrap()
                            .to_string();
                        Type { nullable: true, base: BaseType::Named(Name::new(nm)) }
                    }
                    _ => panic!(),
                }
            } else if let Some(reference) = schema.reference {
                let nm = reference.split('/').last().unwrap().to_string();
                Type { nullable: true, base: BaseType::Named(Name::new(nm)) }
            } else {
                Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
            }
        }
    }
}

fn get_instace_type(typ: &InstanceType) -> String {
    match typ {
        &InstanceType::Integer => "Int".to_string(),
        _ => format!("{:?}", typ),
    }
}

pub trait DirectiveDefinition {
    fn into_schemars() -> RootSchema
    where
        Self: JsonSchema,
    {
        schemars::schema_for!(Self)
    }

    fn directive_definition(generated_types: &mut HashSet<String>) -> Vec<TypeSystemDefinition>;

    fn into_directive_definition(
        root_schema: RootSchema,
        attrs: Attrs,
        generated_types: &mut HashSet<String>,
    ) -> Vec<TypeSystemDefinition> {
        let mut service_doc_definitions = vec![];
        let definitions: BTreeMap<String, Schema> = root_schema.definitions;
        let schema: SchemaObject = root_schema.schema;
        let description = get_description(&schema);

        for (name, schema) in definitions.iter() {
            if generated_types.contains(name) {
                continue;
            }
            let enum_values = get_enum_values(schema);
            if enum_values.is_some() {
                service_doc_definitions.push(generate_enum_definition(enum_values, name));
                generated_types.insert(name.to_string());
            } else {
                service_doc_definitions.push(generate_input_definition(
                    schema.clone().into_object(),
                    name,
                ));
                generated_types.insert(name.to_string());
            }
        }

        let directve_definition = TypeSystemDefinition::Directive(pos(
            async_graphql_parser::types::DirectiveDefinition {
                description: description.map(|inner| pos(inner.clone())),
                name: pos(Name::new(attrs.name)),
                arguments: generate_fields_definition(&schema),
                is_repeatable: attrs.repeatable,
                locations: attrs
                    .locations
                    .into_iter()
                    .map(|val| pos(to_directive_location(val)))
                    .collect(),
            },
        ));
        service_doc_definitions.push(directve_definition);
        service_doc_definitions
    }
}

pub trait ScalarDefinition {
    fn into_schemars() -> RootSchema
    where
        Self: JsonSchema,
    {
        schemars::schema_for!(Self)
    }

    fn scalar_definition() -> TypeSystemDefinition;

    fn into_scalar_definition(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = get_description(&schema);
        TypeSystemDefinition::Type(pos(TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::Scalar,
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }))
    }
}

pub trait InputDefinition {
    fn into_schemars() -> RootSchema
    where
        Self: JsonSchema,
    {
        schemars::schema_for!(Self)
    }

    fn input_definition() -> TypeSystemDefinition;

    fn into_input_definition(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = get_description(&schema);

        TypeSystemDefinition::Type(pos(TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::InputObject(InputObjectType {
                fields: generate_fields_definition(&schema),
            }),
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }))
    }
}

pub trait DocumentDefinition {
    fn definition(doc: ServiceDocument, generated_types: &mut HashSet<String>) -> ServiceDocument;
}

pub struct ServiceDocumentBuilder {
    definitions: Vec<TypeSystemDefinition>,
}

impl ServiceDocumentBuilder {
    pub fn new() -> Self {
        Self { definitions: vec![] }
    }

    pub fn add_directive(
        mut self,
        definitions: Vec<TypeSystemDefinition>,
    ) -> ServiceDocumentBuilder {
        self.definitions.extend(definitions);
        self
    }

    pub fn add_scalar(mut self, definitions: TypeSystemDefinition) -> ServiceDocumentBuilder {
        self.definitions.push(definitions);
        self
    }

    pub fn add_input(mut self, definitions: TypeSystemDefinition) -> ServiceDocumentBuilder {
        self.definitions.push(definitions);
        self
    }

    pub fn build(self) -> ServiceDocument {
        ServiceDocument { definitions: self.definitions }
    }
}

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;

    use super::*;

    #[derive(JsonSchema)]
    enum Schema {
        Obj(String),
        Str,
        Any,
    }

    #[derive(JsonSchema)]
    struct Inpt2Dummy {
        field1: String,
        field2: i32,
        schema: Schema,
    }

    #[derive(JsonSchema)]
    struct InputDummy {
        field1: String,
        field2: Option<String>,
        field3: Inpt2Dummy,
    }

    #[derive(JsonSchema)]
    enum EnumDummy {
        Variant1,
        Variant2,
    }

    #[derive(JsonSchema)]
    struct DirectiveDummy {
        field1: i32,
        field2: Option<i32>,
        field3: Vec<i32>,
        enum_dummy: EnumDummy,
        input_dummy: Vec<InputDummy>,
    }

    impl DocumentDefinition for DirectiveDummy {
        fn definition(
            mut doc: ServiceDocument,
            generated_types: &mut HashSet<String>,
        ) -> ServiceDocument {
            let schema: RootSchema = Self::into_schemars();
            doc.definitions.extend(Self::into_directive_definition(
                schema,
                Attrs { name: "DirectiveDummy", repeatable: false, locations: vec![] },
                generated_types,
            ));
            doc
        }
    }

    #[test]
    fn it_works_for_to_directives() {
        let service_doc = ServiceDocument { definitions: vec![] };
        let mut generated_types = HashSet::new();
        to_service_doc::<DirectiveDummy>(service_doc, &mut generated_types);
        assert_eq!(1, 2);
    }
}
