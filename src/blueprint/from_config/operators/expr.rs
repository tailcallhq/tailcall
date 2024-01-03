use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, ExprEffect, Field};
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

struct CompilationContext<'a> {
  config_field: &'a config::Field,
  operation_type: &'a config::GraphQLOperationType,
  config: &'a config::Config,
}

fn compile_effect(context: &CompilationContext, expr: ExprEffect) -> Valid<Expression, String> {
  let operation_type = context.operation_type;
  let config = context.config;
  let config_field = context.config_field;

  match expr {
    ExprEffect::Http(http) => compile_http(config, config_field, &http),
    ExprEffect::Const(const_field) => compile_const(config, config_field, &const_field),
    ExprEffect::GraphQL(graphql) => compile_graphql(config, operation_type, &graphql),
    ExprEffect::Grpc(grpc) => compile_grpc(CompileGrpc { config, operation_type, field: config_field, grpc: &grpc }),
  }
}

struct CompileIf<'a> {
  context: &'a CompilationContext<'a>,
  condition: Box<ExprBody>,
  then: Box<ExprBody>,
  els: Box<ExprBody>,
}

fn compile_if(input: CompileIf) -> Valid<Expression, String> {
  let context = input.context;
  let condition = input.condition;
  let then = input.then;
  let els = input.els;

  compile(context, *condition)
    .map(Box::new)
    .zip(compile(context, *then).map(Box::new))
    .zip(compile(context, *els).map(Box::new))
    .map(|((condition, then), els)| Expression::If { condition, then, els })
}

fn compile(context: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  match expr {
    ExprBody::If { condition, then, els } => compile_if(CompileIf { context, condition, then, els }),
    ExprBody::Effect(effect) => compile_effect(context, effect),
  }
}

pub fn update_expr(
  operation_type: &config::GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, _, _), b_field| {
    let Some(expr) = &field.expr else {
      return Valid::succeed(b_field);
    };

    let context = CompilationContext { config, operation_type, config_field: field };

    compile(&context, expr.body.clone()).map(|compiled| b_field.resolver(Some(compiled)))
  })
}
