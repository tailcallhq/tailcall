use async_graphql_parser::types::*;
use async_graphql_parser::{Pos, Positioned};
use async_graphql_value::Name;
use schemars::schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec};
use tailcall::core::config::cors::Cors;
use tailcall::core::config::headers::Headers;
use tailcall::core::config::{
    AddField, Alias, Apollo, Batch, Cache, Call, Config, Expr, GraphQL, Grpc, Http, KeyValue, Link,
    Modify, Omit, OtlpExporter, PrometheusExporter, PrometheusFormat, Protected, Proxy,
    ScriptOptions, Server, StdoutExporter, Step, Tag, Telemetry, TelemetryExporter, Upstream, JS,
};
use tailcall::core::scalar::{
    Bytes, Date, Email, Empty, Int128, Int16, Int32, Int64, Int8, PhoneNumber, UInt128, UInt16,
    UInt32, UInt64, UInt8, Url, JSON,
};

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn first_char_to_lowercase(name: &str) -> String {
    let mut chars = name.chars();
    let first_char: String = match chars.next() {
        Some(first_char) => first_char.to_lowercase().collect(),
        None => String::new(),
    };

    format!("{}{}", first_char, chars.collect::<String>())
}

trait DocumentDefinition {
    fn definition(doc: ServiceDocument) -> ServiceDocument;

    fn to_directive(root_schema: RootSchema, name: &str) -> DirectiveDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);
        DirectiveDefinition {
            description: description.map(|inner| pos(inner.clone())),
            name: pos(Name::new(name)),
            arguments: Self::build_arguments(&schema),
            is_repeatable: false,
            locations: vec![],
        }
    }

    fn to_scalar(root_schema: RootSchema, name: &str) -> TypeDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);
        TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::Scalar,
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }
    }

    fn to_input(root_schema: RootSchema, name: &str) -> TypeDefinition {
        let schema: SchemaObject = root_schema.schema;
        let description = Self::get_description(&schema);
        TypeDefinition {
            name: pos(Name::new(name)),
            kind: TypeKind::InputObject(InputObjectType { fields: Self::build_arguments(&schema) }),
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }
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

macro_rules! impl_doc_definition {
    ($ty:ty, "Directive") => {
        impl DocumentDefinition for $ty {
            fn definition(mut doc: ServiceDocument) -> ServiceDocument {
                let root_schema = schemars::schema_for!(Self);
                doc.definitions
                    .push(TypeSystemDefinition::Directive(pos(Self::to_directive(
                        root_schema.clone(),
                        first_char_to_lowercase(stringify!($ty)).as_str(),
                    ))));
                doc
            }
        }
    };

    ($ty:ty, "Input") => {
        impl DocumentDefinition for $ty {
            fn definition(mut doc: ServiceDocument) -> ServiceDocument {
                let root_schema = schemars::schema_for!(Self);
                doc.definitions
                    .push(TypeSystemDefinition::Type(pos(Self::to_input(
                        root_schema.clone(),
                        stringify!($ty),
                    ))));
                doc
            }
        }
    };

    ($ty:ty, "DirectiveWithInput") => {
        impl DocumentDefinition for $ty {
            fn definition(mut doc: ServiceDocument) -> ServiceDocument {
                let root_schema = schemars::schema_for!(Self);
                doc.definitions
                    .push(TypeSystemDefinition::Directive(pos(Self::to_directive(
                        root_schema.clone(),
                        first_char_to_lowercase(stringify!($ty)).as_str(),
                    ))));

                doc.definitions
                    .push(TypeSystemDefinition::Type(pos(Self::to_input(
                        root_schema.clone(),
                        stringify!($ty),
                    ))));
                doc
            }
        }
    };

    ($ty:ty, "Scalar") => {
        impl DocumentDefinition for $ty {
            fn definition(mut doc: ServiceDocument) -> ServiceDocument {
                let root_schema = schemars::schema_for!(Self);
                doc.definitions
                    .push(TypeSystemDefinition::Type(pos(Self::to_scalar(
                        root_schema,
                        stringify!($ty),
                    ))));
                doc
            }
        }
    };
}

impl_doc_definition!(Batch, "Input");
impl_doc_definition!(Apollo, "Input");
impl_doc_definition!(Cors, "Input");
impl_doc_definition!(Headers, "Input");
impl_doc_definition!(KeyValue, "Input");
impl_doc_definition!(OtlpExporter, "Input");
impl_doc_definition!(PrometheusExporter, "Input");
impl_doc_definition!(PrometheusFormat, "Input");
impl_doc_definition!(Proxy, "Input");
impl_doc_definition!(ScriptOptions, "Input");
impl_doc_definition!(StdoutExporter, "Input");
impl_doc_definition!(Step, "Input");
impl_doc_definition!(TelemetryExporter, "Input");
//impl_doc_definition!(JsonSchema, "Input");

impl_doc_definition!(Grpc, "DirectiveWithInput");
impl_doc_definition!(GraphQL, "DirectiveWithInput");
impl_doc_definition!(Cache, "DirectiveWithInput");
impl_doc_definition!(Http, "DirectiveWithInput");
impl_doc_definition!(Expr, "DirectiveWithInput");
impl_doc_definition!(Modify, "DirectiveWithInput");
impl_doc_definition!(Telemetry, "DirectiveWithInput");
impl_doc_definition!(Server, "Directive");
impl_doc_definition!(Link, "Directive");
impl_doc_definition!(Upstream, "Directive");
impl_doc_definition!(Call, "Directive");
impl_doc_definition!(AddField, "Directive");
impl_doc_definition!(Omit, "Directive");
impl_doc_definition!(Protected, "Directive");
impl_doc_definition!(JS, "DirectiveWithInput");
impl_doc_definition!(Tag, "Directive");
impl_doc_definition!(Alias, "Directive");

impl_doc_definition!(PhoneNumber, "Scalar");
impl_doc_definition!(Date, "Scalar");
impl_doc_definition!(Url, "Scalar");
impl_doc_definition!(JSON, "Scalar");
impl_doc_definition!(Empty, "Scalar");
impl_doc_definition!(Int8, "Scalar");
impl_doc_definition!(Int16, "Scalar");
impl_doc_definition!(Int32, "Scalar");
impl_doc_definition!(Int64, "Scalar");
impl_doc_definition!(Int128, "Scalar");
impl_doc_definition!(UInt8, "Scalar");
impl_doc_definition!(UInt16, "Scalar");
impl_doc_definition!(UInt32, "Scalar");
impl_doc_definition!(UInt64, "Scalar");
impl_doc_definition!(UInt128, "Scalar");
impl_doc_definition!(Email, "Scalar");
impl_doc_definition!(Bytes, "Scalar");

impl DocumentDefinition for Config {
    fn definition(doc: ServiceDocument) -> ServiceDocument {
        let mut doc = doc;
        macro_rules! to_service_doc {
            ($ty:ty) => {
                doc = <$ty>::definition(doc)
            };
        }

        // directives
        to_service_doc!(AddField);
        to_service_doc!(Alias);
        to_service_doc!(Cache);
        to_service_doc!(Call);
        to_service_doc!(Expr);
        to_service_doc!(GraphQL);
        to_service_doc!(Grpc);
        to_service_doc!(Http);
        to_service_doc!(JS);
        to_service_doc!(Link);
        to_service_doc!(Modify);
        to_service_doc!(Omit);
        to_service_doc!(Protected);
        to_service_doc!(Server);
        to_service_doc!(Tag);
        to_service_doc!(Telemetry);
        to_service_doc!(Upstream);

        // default scalars
        to_service_doc!(Bytes);
        to_service_doc!(Email);
        to_service_doc!(Date);
        to_service_doc!(PhoneNumber);
        to_service_doc!(Url);
        to_service_doc!(JSON);
        to_service_doc!(Empty);
        to_service_doc!(Int8);
        to_service_doc!(Int16);
        to_service_doc!(Int32);
        to_service_doc!(Int64);
        to_service_doc!(Int128);
        to_service_doc!(UInt8);
        to_service_doc!(UInt16);
        to_service_doc!(UInt32);
        to_service_doc!(UInt64);
        to_service_doc!(UInt128);

        // inputs
        to_service_doc!(Batch);
        to_service_doc!(Apollo);
        to_service_doc!(Cors);
        to_service_doc!(Headers);
        to_service_doc!(KeyValue);
        to_service_doc!(OtlpExporter);
        to_service_doc!(PrometheusExporter);
        to_service_doc!(PrometheusFormat);
        to_service_doc!(Proxy);
        to_service_doc!(ScriptOptions);
        to_service_doc!(StdoutExporter);
        to_service_doc!(Step);
        to_service_doc!(TelemetryExporter);

        //to_service_doc!(JsonSchema);

        doc
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_works() {}
}
