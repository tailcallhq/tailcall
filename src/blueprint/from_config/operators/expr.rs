use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, ExprEffect, Field};
use crate::directive::DirectiveCodec;
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

struct CompilationContext<'a> {
  config_field: &'a config::Field,
  operation_type: &'a config::GraphQLOperationType,
  ty: &'a config::Type,
  name: &'a str,
  config: &'a config::Config,
}

fn setup_field(field: &Field, expr: ExprEffect) -> Field {
  let copy = field.clone();
  match expr {
    ExprEffect::Http(http) => copy.http(http.clone()),
    ExprEffect::Const(const_field) => copy.const_field(const_field.clone()),
    ExprEffect::Grpc(grpc) => copy.grpc(grpc.clone()),
    ExprEffect::GraphQL(graphql) => copy.graphql(graphql.clone()),
  }
}

// TODO: Extract out just the resolver construction from operators instead of doing the whole TryFold
fn compile_effect(context: &CompilationContext, expr: ExprEffect) -> Valid<Expression, String> {
  let operation_type = context.operation_type;
  let config = context.config;
  let ty = context.ty;
  let name = context.name;
  let config_field = context.config_field;

  if let ExprEffect::Http(http) = &expr {
    compile_http(config, config_field, http)
  } else {
    let field_with_expr = setup_field(config_field, expr);
    let field_with_resolver = update_const_field()
      .trace(config::Const::trace_name().as_str())
      .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
      .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
      .try_fold(&(config, &field_with_expr, ty, name), FieldDefinition::default());

    field_with_resolver.and_then(|f| match f.resolver {
      Some(resolver) => Valid::succeed(resolver),
      None => Valid::fail("failed to compile effect".to_string()),
    })
  }
}

fn compile_if(
  context: &CompilationContext,
  condition: Box<ExprBody>,
  then: Box<ExprBody>,
  els: Box<ExprBody>,
) -> Valid<Expression, String> {
  compile(context, *condition)
    .map(Box::new)
    .zip(compile(context, *then).map(Box::new))
    .zip(compile(context, *els).map(Box::new))
    .map(|((condition, then), els)| Expression::If { condition, then, els })
}

fn compile(context: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  match expr {
    ExprBody::If { condition, then, els } => compile_if(context, condition, then, els),
    ExprBody::Effect(effect) => compile_effect(context, effect),
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

      let context = CompilationContext { ty, name, config, operation_type, config_field: field };

      compile(&context, expr.body.clone()).map(|compiled| b_field.resolver(Some(compiled)))
    },
  )
}
