use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, Field};
use crate::directive::DirectiveCodec;
use crate::try_fold::TryFold;
use crate::valid::Valid;

fn setup_field(field: &Field, expr: &ExprBody) -> Field {
  let copy = field.clone();
  match expr {
    ExprBody::Http(http) => copy.http(http.clone()),
    ExprBody::Const(const_field) => copy.const_field(const_field.clone()),
    ExprBody::Grpc(grpc) => copy.grpc(grpc.clone()),
    ExprBody::GraphQL(graphql) => copy.graphql(graphql.clone()),
  }
}

pub fn update_expr(
  operation_type: &config::GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    |(config, field, ty, name), b_field| {
      let Some(expr) = &field.expr else {
        return Valid::succeed(b_field);
      };

      let field_with_expr = setup_field(field, &expr.body);
      let field_with_resolver = update_http()
        .trace(config::Http::trace_name().as_str())
        .and(update_const_field().trace(config::Const::trace_name().as_str()))
        .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
        .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
        .try_fold(&(config, &field_with_expr, ty, name), b_field);
      field_with_resolver
    },
  )
}
