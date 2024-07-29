use convert_case::{Case, Casing};

use super::http_directive_generator::HttpDirectiveGenerator;
use crate::core::config::{Arg, Config, Field, Type};
use crate::core::generator::json::types_generator::TypeGenerator;
use crate::core::generator::{NameGenerator, OperationType, RequestSample};
use crate::core::http::Method;
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
            list: request_sample.response().is_array(),
            type_of: root_type.to_owned(),
            ..Default::default()
        };

        // generate required http directive.
        let http_directive_gen = HttpDirectiveGenerator::new(request_sample.url());
        field.http = Some(http_directive_gen.generate_http_directive(&mut field));

        // we provide default names for operation name, then we change it in subsequent
        // steps.
        let operation_name = match &request_sample.operation_type() {
            OperationType::Query => "Query",
            OperationType::Mutation { body } => {
                // generate the input type.
                let root_ty = TypeGenerator::new(name_generator).generate_types(body, &mut config);
                // add input type to field.
                let arg_name = root_ty.to_case(Case::Camel);
                if let Some(http_) = &mut field.http {
                    http_.body = Some(format!("{{{{.args.{}}}}}", arg_name.clone()));
                    http_.method = Method::POST;
                }
                field.args.insert(
                    arg_name,
                    Arg { type_of: root_ty, ..Default::default() },
                );
                "Mutation"
            }
        };

        // if type is already present, then append the new field to it else create one.
        if let Some(type_) = config.types.get_mut(operation_name) {
            type_
                .fields
                .insert(request_sample.field_name().to_owned(), field);
        } else {
            let mut ty = Type::default();
            ty.fields
                .insert(request_sample.field_name().to_owned(), field);
            config.types.insert(operation_name.to_owned(), ty);
        }

        Valid::succeed(config)
    }
}
