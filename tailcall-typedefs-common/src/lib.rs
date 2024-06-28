use async_graphql_parser::types::*;
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use schemars::schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec};

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

pub trait DocumentDefinition {
    fn definition(doc: ServiceDocument) -> ServiceDocument;

    fn to_doc_directive(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);

        TypeSystemDefinition::Directive(pos(DirectiveDefinition {
            description: description.map(|inner| pos(inner.clone())),
            name: pos(Name::new(name)),
            arguments: Self::build_arguments(&schema),
            is_repeatable: false,
            locations: vec![],
        }))
    }

    fn to_doc_scalar(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);
        TypeSystemDefinition::Type(pos(TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::Scalar,
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }))
    }

    fn to_doc_input(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);
        TypeSystemDefinition::Type(pos(TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::InputObject(InputObjectType { fields: Self::build_arguments(&schema) }),
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }))
    }

    fn build_arguments(schema: &SchemaObject) -> Vec<Positioned<InputValueDefinition>> {
        let mut arguments = vec![];
        if let Some(properties) = schema
            .object
            .as_ref()
            .map(|object| object.properties.clone())
        {
            for (name, property) in properties.into_iter() {
                let property = property.into_object();
                let description = Self::get_description(&property);
                let definition = pos(InputValueDefinition {
                    description: description.map(|inner| pos(inner.to_owned())),
                    name: pos(Name::new(name.to_owned())),
                    ty: pos(Self::get_type(name, property.clone())),
                    default_value: None,
                    directives: Vec::new(),
                });

                arguments.push(definition);
            }
        }

        arguments
    }

    fn get_type(name: String, schema: SchemaObject) -> Type {
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
                    base: BaseType::Named(Name::new(Self::get_instace_type(&*typ))),
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
                    nullable: false,
                    base: BaseType::Named(Name::new(Self::get_instace_type(typ.first().unwrap()))),
                }
            }
            _ => {
                if let Some(arr_valid) = schema.array {
                    if let Some(SingleOrVec::Single(schema)) = arr_valid.items {
                        return Type {
                            nullable: true,
                            base: BaseType::List(Box::new(Self::get_type(
                                name,
                                schema.into_object(),
                            ))),
                        };
                    } else if let Some(SingleOrVec::Vec(schemas)) = arr_valid.items {
                        return Type {
                            nullable: false,
                            base: BaseType::List(Box::new(Self::get_type(
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
                        return Type { nullable: false, base: BaseType::Named(Name::new("JSON")) };
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
                    Type { nullable: false, base: BaseType::Named(Name::new(nm)) }
                } else {
                    Type { nullable: false, base: BaseType::Named(Name::new("JSON")) }
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

    fn get_description<'a>(schema: &'a SchemaObject) -> Option<&'a String> {
        schema
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.description.as_ref())
    }
}
