use convert_case::{Case, Casing};

use super::http_directive_generator::HttpDirectiveGenerator;
use crate::core::config::{Arg, Config, Field, GraphQLOperationType, Type};
use crate::core::generator::json::types_generator::TypeGenerator;
use crate::core::generator::{NameGenerator, RequestSample};
use crate::core::valid::Valid;

pub struct OperationTypeGenerator;

impl OperationTypeGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        request_sample: &RequestSample,
        root_type: &str,
        name_generator: &NameGenerator,
        mut config: Config,
    ) -> Valid<Config, String> {
        let mut field = Field {
            list: request_sample.response.is_array(),
            type_of: root_type.to_owned(),
            ..Default::default()
        };

        // generate required http directive.
        let http_directive_gen = HttpDirectiveGenerator::new(&request_sample.url);
        field.http = Some(http_directive_gen.generate_http_directive(&mut field));

        if let GraphQLOperationType::Mutation = request_sample.operation_type {
            // generate the input type.
            let root_ty = TypeGenerator::new(name_generator)
                .generate_types(&request_sample.body, &mut config);
            // add input type to field.
            let arg_name = format!("{}Input", request_sample.field_name).to_case(Case::Camel);
            if let Some(http_) = &mut field.http {
                http_.body = Some(format!("{{{{.args.{}}}}}", arg_name.clone()));
                http_.method = request_sample.method.to_owned();
            }
            field
                .args
                .insert(arg_name, Arg { type_of: root_ty, ..Default::default() });
        }

        // if type is already present, then append the new field to it else create one.
        let req_op = request_sample
            .operation_type
            .to_string()
            .to_case(Case::Pascal);
        if let Some(type_) = config.types.get_mut(req_op.as_str()) {
            type_
                .fields
                .insert(request_sample.field_name.to_owned(), field);
        } else {
            let mut ty = Type::default();
            ty.fields
                .insert(request_sample.field_name.to_owned(), field);
            config.types.insert(req_op.to_owned(), ty);
        }

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::OperationTypeGenerator;
    use crate::core::config::{Config, Field, GraphQLOperationType, Type};
    use crate::core::generator::{NameGenerator, RequestSample};
    use crate::core::http::Method;
    use crate::core::valid::Validator;

    #[test]
    fn test_query() {
        let sample = RequestSample::new(
            "https://jsonplaceholder.typicode.com/comments?postId=1"
                .parse()
                .unwrap(),
            Method::GET,
            serde_json::Value::Null,
            serde_json::Value::Null,
            "postComments",
            GraphQLOperationType::Query,
        );
        let config = Config::default();
        let config = OperationTypeGenerator
            .generate(&sample, "T44", &NameGenerator::new("Input"), config)
            .to_result()
            .unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_append_field_if_operation_type_exists() {
        let sample = RequestSample::new(
            "https://jsonplaceholder.typicode.com/comments?postId=1"
                .parse()
                .unwrap(),
            Method::GET,
            serde_json::Value::Null,
            serde_json::Value::Null,
            "postComments",
            GraphQLOperationType::Query,
        );
        let mut config = Config::default();
        let mut fields = BTreeMap::default();
        fields.insert(
            "post".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );

        let type_ = Type { fields, ..Default::default() };
        config.types.insert("Query".to_owned(), type_);

        let config = OperationTypeGenerator
            .generate(&sample, "T44", &NameGenerator::new("Input"), config)
            .to_result()
            .unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_mutation() {
        let body = r#"
            {
            "id": 1,
            "title": "tailcall: modern graphQL runtime",
            "body": "modern graphQL runtime",
            "userId": 1
            }
        "#;

        let sample = RequestSample::new(
            "https://jsonplaceholder.typicode.com/posts"
                .parse()
                .unwrap(),
            Method::POST,
            serde_json::from_str(body).unwrap(),
            serde_json::Value::Null,
            "postComments",
            GraphQLOperationType::Mutation,
        );
        let config = Config::default();
        let config = OperationTypeGenerator
            .generate(&sample, "T44", &NameGenerator::new("Input"), config)
            .to_result()
            .unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }
}
