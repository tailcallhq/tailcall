use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tailcall::core::config::{
    AddField, Alias, Cache, Call, Expr, GraphQL, Grpc, Http, Link, Modify, Omit, Protected, Server,
    Tag, Telemetry, Upstream, JS,
};
use tailcall::core::scalar::{
    Bytes, Date, Email, Empty, Int128, Int16, Int32, Int64, Int8, PhoneNumber, UInt128, UInt16,
    UInt32, UInt64, UInt8, Url, JSON,
};
use tailcall::core::FileIO;
use tailcall_typedefs_common::directive_definition::DirectiveDefinition;
use tailcall_typedefs_common::input_definition::InputDefinition;
use tailcall_typedefs_common::scalar_definition::ScalarDefinition;
use tailcall_typedefs_common::ServiceDocumentBuilder;

static GRAPHQL_SCHEMA_FILE: &str = "generated/.tailcallrc";

pub async fn update_gql(file_io: Arc<dyn FileIO>) -> Result<()> {
    let mut generated_types: HashSet<String> = HashSet::new();
    let builder = ServiceDocumentBuilder::new();

    let generated_types_mut = &mut generated_types;
    let doc = builder
        .add_directive(AddField::directive_definition(generated_types_mut))
        .add_directive(Alias::directive_definition(generated_types_mut))
        .add_directive(Cache::directive_definition(generated_types_mut))
        .add_directive(Call::directive_definition(generated_types_mut))
        .add_directive(Expr::directive_definition(generated_types_mut))
        .add_directive(GraphQL::directive_definition(generated_types_mut))
        .add_directive(Grpc::directive_definition(generated_types_mut))
        .add_directive(Http::directive_definition(generated_types_mut))
        .add_directive(JS::directive_definition(generated_types_mut))
        .add_directive(Link::directive_definition(generated_types_mut))
        .add_directive(Modify::directive_definition(generated_types_mut))
        .add_directive(Omit::directive_definition(generated_types_mut))
        .add_directive(Protected::directive_definition(generated_types_mut))
        .add_directive(Server::directive_definition(generated_types_mut))
        .add_directive(Tag::directive_definition(generated_types_mut))
        .add_directive(Telemetry::directive_definition(generated_types_mut))
        .add_directive(Upstream::directive_definition(generated_types_mut))
        .add_input(GraphQL::input_definition())
        .add_input(Grpc::input_definition())
        .add_input(Http::input_definition())
        .add_input(Expr::input_definition())
        .add_input(JS::input_definition())
        .add_input(Modify::input_definition())
        .add_input(Cache::input_definition())
        .add_input(Telemetry::input_definition())
        .add_scalar(Bytes::scalar_definition())
        .add_scalar(Date::scalar_definition())
        .add_scalar(Email::scalar_definition())
        .add_scalar(Empty::scalar_definition())
        .add_scalar(Int128::scalar_definition())
        .add_scalar(Int16::scalar_definition())
        .add_scalar(Int32::scalar_definition())
        .add_scalar(Int64::scalar_definition())
        .add_scalar(Int8::scalar_definition())
        .add_scalar(JSON::scalar_definition())
        .add_scalar(PhoneNumber::scalar_definition())
        .add_scalar(UInt128::scalar_definition())
        .add_scalar(UInt16::scalar_definition())
        .add_scalar(UInt32::scalar_definition())
        .add_scalar(UInt64::scalar_definition())
        .add_scalar(UInt8::scalar_definition())
        .add_scalar(Url::scalar_definition())
        .build();

    let mut path = PathBuf::from(GRAPHQL_SCHEMA_FILE);
    path.set_extension("graphql");
    file_io
        .write(
            path.to_str().ok_or(anyhow!("Unable to determine path"))?,
            tailcall::core::document::print(doc).as_bytes(),
        )
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use std::collections::HashSet;

    use schemars::JsonSchema;
    use tailcall_typedefs_common::ServiceDocumentBuilder;

    use tailcall_typedefs_common::directive_definition::{into_directive_definition, Attrs, DirectiveDefinition};
    use tailcall_typedefs_common::scalar_definition::ScalarDefinition;

    #[derive(JsonSchema)]
    struct FooScalar(String);

    impl ScalarDefinition for FooScalar {
        fn scalar_definition() -> async_graphql_parser::types::TypeSystemDefinition {
            let root_schema = Self::into_schemars();
            into_scalar_definition(root_schema, "FooScalar")
        }
    }

    #[derive(JsonSchema)]
    struct ComplexDirective {
        field1: i32,
        enum_field: FooEnum,
        custom_type: FooType
    }

    impl DirectiveDefinition for ComplexDirective {
        fn directive_definition(generated_types: &mut HashSet<String>) -> Vec<async_graphql_parser::types::TypeSystemDefinition> {
            let root_schema = Self::into_schemars();

            into_directive_definition(root_schema, Attrs {
                name: "ComplexDirective",
                repeatable: true,
                locations: vec!["Schema"],
                is_lowercase_name: false
            }, generated_types)

        }
    }

    #[derive(JsonSchema)]
    enum FooEnum {
        Variant,
        Variant2,
        Variat3
    }

    #[derive(JsonSchema)]
    struct FooType {
        field1: i32,
        field2: Option<i32>,
        field3: Vec<String>,
        inner_type: BarType
    }

    #[derive(JsonSchema)]
    struct BarType {
        field2: BazType
    }

    #[derive(JsonSchema)]
    struct BazType {
        field: i32
    }

    #[test]
    fn it_works_for_into_scalar() {
        let builder = ServiceDocumentBuilder::new();
        let doc = builder.add_scalar(FooScalar::scalar_definition()).build();
        let actual = tailcall::core::document::print(doc);
        let expected = "scalar FooScalar".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_works_for_into_directive_with_complex_type() {
        let builder = ServiceDocumentBuilder::new();
        let doc = builder.add_directive(ComplexDirective::directive_definition(&mut HashSet::new())).build();
        let actual = tailcall::core::document::print(doc);
        let expected = "directive @complexDirective(\n  custom_type: FooType\n  enum_field: FooEnum\n  field1: Int!\n) repeatable on SCHEMA\n\ninput BarType {\n  field2: BazType\n}\n\ninput BazType {\n  field: Int!\n}\n\ninput FooType {\n  field1: Int!\n  field2: Int\n  field3: [String!]\n  inner_type: BarType\n}\n\nenum FooEnum {\n  Variant\n  Variant2\n  Variat3\n}".to_string();

        assert_eq!(actual, expected);
    }
}
