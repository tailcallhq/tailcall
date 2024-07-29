use super::http_directive_generator::HttpDirectiveGenerator;
use crate::core::config::{Arg, Config, Field, Type};
use crate::core::generator::json::types_generator::TypeGenerator;
use crate::core::generator::{NameGenerator, OperationType, RequestSample};
use crate::core::http::Method;
use crate::core::valid::Valid;

pub struct OperationTypeGenerator;

impl OperationTypeGenerator {
    pub fn generate(
        &self,
        request_sample: &RequestSample,
        root_type: &str,
        mut config: Config,
    ) -> Valid<Config, String> {
        let mut field = Field {
            list: request_sample.response().is_array(),
            type_of: root_type.to_owned(),
            ..Default::default()
        };

        // if type is already present, then append the new field to it else create one.
        // generate required http directive.
        let http_directive_gen = HttpDirectiveGenerator::new(request_sample.url());
        field.http = Some(http_directive_gen.generate_http_directive(&mut field));

        match &request_sample.operation_type() {
            OperationType::Query => {
                if let Some(type_) = config.types.get_mut("Query") {
                    type_
                        .fields
                        .insert(request_sample.field_name().to_owned(), field);
                } else {
                    let mut ty = Type::default();
                    ty.fields
                        .insert(request_sample.field_name().to_owned(), field);
                    config.types.insert("Query".to_owned(), ty);
                }
            }
            OperationType::Mutation { body: _body } => {
                // generate the input type.
                let root_ty = TypeGenerator::new(&NameGenerator::new("Input"))
                    .generate_types(_body, &mut config);
                if let Some(http_) = &mut field.http {
                    http_.body = Some(format!("{{{{.args.{}}}}}", root_type));
                    http_.method = Method::POST;
                }
                field.args.insert(
                    root_type.to_owned(),
                    Arg { type_of: root_ty, ..Default::default() },
                );

                // if type is already present, then append the new field to it else create one.
                if let Some(type_) = config.types.get_mut("Mutation") {
                    type_
                        .fields
                        .insert(request_sample.field_name().to_owned(), field);
                } else {
                    let mut ty = Type::default();
                    ty.fields
                        .insert(request_sample.field_name().to_owned(), field);
                    config.types.insert("Mutation".to_owned(), ty);
                }
            }
        }

        Valid::succeed(config)
    }
}
