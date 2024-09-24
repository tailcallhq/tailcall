use async_graphql::parser::types::ServiceDocument;
use tailcall::core::config::Config;

pub fn build_service_document() -> ServiceDocument {
    Config::graphql_schema()
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use std::collections::HashSet;

    use schemars::schema::Schema;
    use schemars::JsonSchema;
    use tailcall_typedefs_common::directive_definition::{
        into_directive_definition, Attrs, DirectiveDefinition,
    };
    use tailcall_typedefs_common::scalar_definition::{into_scalar_definition, ScalarDefinition};
    use tailcall_typedefs_common::{into_schemars, ServiceDocumentBuilder};

    #[derive(JsonSchema)]
    struct FooScalar(String);

    impl ScalarDefinition for FooScalar {
        fn scalar_definition() -> async_graphql::parser::types::TypeSystemDefinition {
            let root_schema = into_schemars::<Self>();
            into_scalar_definition(Schema::Object(root_schema.schema), "FooScalar")
        }
    }

    #[derive(JsonSchema)]
    struct ComplexDirective {
        field1: i32,
        enum_field: FooEnum,
        custom_type: FooType,
    }

    impl DirectiveDefinition for ComplexDirective {
        fn directive_definition(
            generated_types: &mut HashSet<String>,
        ) -> Vec<async_graphql::parser::types::TypeSystemDefinition> {
            let root_schema = into_schemars::<Self>();

            into_directive_definition(
                root_schema,
                Attrs {
                    name: "ComplexDirective",
                    repeatable: true,
                    locations: vec!["Schema"],
                    is_lowercase_name: false,
                },
                generated_types,
            )
        }
    }

    #[derive(JsonSchema)]
    enum FooEnum {
        Variant,
        Variant2,
        Variat3,
    }

    #[derive(JsonSchema)]
    struct FooType {
        field1: i32,
        field2: Option<i32>,
        field3: Vec<String>,
        inner_type: BarType,
    }

    #[derive(JsonSchema)]
    struct BarType {
        field2: BazType,
    }

    #[derive(JsonSchema)]
    struct BazType {
        field: i32,
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
        let doc = builder
            .add_directive(ComplexDirective::directive_definition(&mut HashSet::new()))
            .build();
        let actual = tailcall::core::document::print(doc);
        let expected = "directive @complexDirective(\n  custom_type: FooType\n  enum_field: FooEnum\n  field1: Int!\n) repeatable on SCHEMA\n\ninput BarType {\n  field2: BazType\n}\n\ninput BazType {\n  field: Int!\n}\n\ninput FooType {\n  field1: Int!\n  field2: Int\n  field3: [String!]\n  inner_type: BarType\n}\n\nenum FooEnum {\n  Variant\n  Variant2\n  Variat3\n}".to_string();

        assert_eq!(actual, expected);
    }
}
