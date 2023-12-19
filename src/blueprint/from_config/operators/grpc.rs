use std::path::Path;

use crate::blueprint::FieldDefinition;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field, GraphQLOperationType, Grpc, GrpcBatchOperation};
use crate::grpc::protobuf::{ProtobufOperation, ProtobufSet};
use crate::grpc::request_template::RequestTemplate;
use crate::lambda::Lambda;
use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{config, helpers};

fn to_url(grpc: &Grpc, config: &Config) -> Valid<Mustache, String> {
  Valid::from_option(
    grpc.base_url.as_ref().or(config.upstream.base_url.as_ref()),
    "No base URL defined".to_string(),
  )
  .and_then(|base_url| {
    let mut base_url = base_url.trim_end_matches('/').to_owned();
    base_url.push('/');
    base_url.push_str(&grpc.service);
    base_url.push('/');
    base_url.push_str(&grpc.method);

    helpers::url::to_url(&base_url)
  })
}

fn to_operations(grpc: &Grpc) -> Valid<(ProtobufOperation, Option<GrpcBatchOperation>), String> {
  Valid::from(
    ProtobufSet::from_proto_file(Path::new(&grpc.proto_path)).map_err(|e| ValidationError::new(e.to_string())),
  )
  .and_then(|set| {
    Valid::from(
      set
        .find_service(&grpc.service)
        .map_err(|e| ValidationError::new(e.to_string())),
    )
  })
  .and_then(|service| {
    let operation = Valid::from(
      service
        .find_operation(&grpc.method)
        .map_err(|e| ValidationError::new(e.to_string())),
    );

    if let Some(batch) = &grpc.batch {
      return operation.zip(Valid::from(
        service
          .find_operation(&batch.method)
          .map(|operation| Some(GrpcBatchOperation { operation, group_by: GroupBy::new(batch.group_by.clone()) }))
          .map_err(|e| ValidationError::new(e.to_string())),
      ));
    }

    operation.zip(Valid::succeed(None))
  })
}

pub fn update_grpc<'a>(
  operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, _type_of, _), b_field| {
      let Some(grpc) = &field.grpc else {
        return Valid::succeed(b_field);
      };

      to_url(grpc, config)
        .zip(to_operations(grpc))
        .zip(helpers::headers::to_headervec(&grpc.headers))
        .zip(helpers::body::to_body(grpc.body.as_deref()))
        .map(|(((url, (operation, batch)), headers), body)| {
          let request_template =
            RequestTemplate { url, headers, operation, body, operation_type: operation_type.to_owned() };

          b_field.resolver(Some(
            Lambda::from_grpc_request_template(request_template, batch).expression,
          ))
        })
    },
  )
}
