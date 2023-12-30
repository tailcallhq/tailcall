use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, Field};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_expr(
  operation_type: &config::GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    |(config, field, ty, name), b_field| {
      let Some(expr) = &field.expr else {
        return Valid::succeed(b_field);
      };

      match &expr.body {
        ExprBody::Http(http) => {
          let field_with_http = (*field).clone().http(http.clone());
          let http_field_def = update_http().try_fold(&(config, &field_with_http, ty, name), b_field);
          http_field_def
        }
        ExprBody::Const(const_field) => {
          let field_with_const = (*field).clone().const_field(const_field.clone());
          let const_field_def = update_const_field().try_fold(&(config, &field_with_const, ty, name), b_field);
          const_field_def
        }
        ExprBody::GraphQL(gql) => {
          let field_with_gql = (*field).clone().graphql(gql.clone());
          let gql_field_def = update_graphql(operation_type).try_fold(&(config, &field_with_gql, ty, name), b_field);
          gql_field_def
        }
        ExprBody::Grpc(grpc) => {
          let field_with_grpc = (*field).clone().grpc(grpc.clone());
          let grpc_field_def = update_grpc(operation_type).try_fold(&(config, &field_with_grpc, ty, name), b_field);
          grpc_field_def
        }
      }
    },
  )
}
