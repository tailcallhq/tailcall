use std::path::PathBuf;

use reqwest::header::{HeaderValue, CONTENT_TYPE};

use crate::blueprint::{to_json_schema_for_args, to_json_schema_for_field, FieldDefinition};
use crate::config::{Config, Field};
use crate::endpoint::Endpoint;
use crate::http::{Method, RequestTemplate};
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{config, helpers};

pub fn update_grpc<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, _type_of, _), b_field| {
      let Some(grpc) = &field.grpc else {
        return Valid::succeed(b_field);
      };
      Valid::from_option(
        grpc.base_url.as_ref().or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
      )
      .zip(helpers::headers::to_headermap(&grpc.headers))
      .and_then(|(base_url, mut header_map)| {
        let mut base_url = base_url.trim_end_matches('/').to_owned();
        base_url.push('/');
        base_url.push_str(grpc.service.clone().as_str());
        base_url.push('/');
        base_url.push_str(grpc.method.clone().as_str());
        header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/grpc"));

        let output_schema = to_json_schema_for_field(field, config);
        let input_schema = to_json_schema_for_args(&field.args, config);

        RequestTemplate::try_from(
          Endpoint::new(base_url.to_string())
            .method(Method::GET)
            .output(output_schema)
            .input(input_schema)
            .body(grpc.body.clone())
            .headers(header_map),
        )
        .map_err(|e| ValidationError::new(e.to_string()))
        .into()
      })
      .map(|req_template| {
        let pathbuf = PathBuf::from(&grpc.proto_path);
        let file = crate::grpc::protobuf::ProtobufSet::from_proto_file(&pathbuf).unwrap();
        let service = crate::grpc::protobuf::ProtobufService::new(&file, grpc.service.as_str()).unwrap();
        let operation = crate::grpc::protobuf::ProtobufOperation::new(&service, grpc.method.as_str()).unwrap();

        b_field.resolver(Some(
          Lambda::from_request_template(req_template.grpc(Some(operation))).expression,
        ))
      })
    },
  )
}
