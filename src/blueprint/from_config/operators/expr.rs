use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, Field};
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

struct CompilationContext<'a> {
  config_field: &'a config::Field,
  operation_type: &'a config::GraphQLOperationType,
  config: &'a config::Config,
}

struct CompileIf<'a> {
  context: &'a CompilationContext<'a>,
  cond: Box<ExprBody>,
  then: Box<ExprBody>,
  els: Box<ExprBody>,
}

struct CompileBinaryOp<'a> {
  context: &'a CompilationContext<'a>,
  lhs: Box<ExprBody>,
  rhs: Box<ExprBody>,
}

fn compile(context: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  let config = context.config;
  let field = context.config_field;
  let operation_type = context.operation_type;

  match expr {
    ExprBody::If { cond, then, els } => compile_if(CompileIf { context, cond, then, els }),
    ExprBody::Http(http) => compile_http(config, field, &http),
    ExprBody::Grpc(grpc) => {
      compile_grpc(CompileGrpc { config, field, operation_type, grpc: &grpc, validate_with_schema: false })
    }
    ExprBody::GraphQL(gql) => compile_graphql(config, operation_type, &gql),
    ExprBody::Const(value) => compile_const(CompileConst { config, field, value: &value, validate_with_schema: false }),
    ExprBody::Concat { lhs, rhs } => compile_concat(CompileBinaryOp { context, lhs, rhs }),
    ExprBody::Intersection { lhs, rhs } => compile_intersection(CompileBinaryOp { context, lhs, rhs }),
  }
}

fn compile_intersection(input: CompileBinaryOp) -> Valid<Expression, String> {
  let context = input.context;
  let lhs = input.lhs;
  let rhs = input.rhs;

  compile(context, *lhs)
    .map(Box::new)
    .zip(compile(context, *rhs).map(Box::new))
    .map(|(lhs, rhs)| Expression::Intersection { lhs, rhs })
}

fn compile_concat(input: CompileBinaryOp) -> Valid<Expression, String> {
  let context = input.context;
  let lhs = input.lhs;
  let rhs = input.rhs;

  compile(context, *lhs)
    .map(Box::new)
    .zip(compile(context, *rhs).map(Box::new))
    .map(|(lhs, rhs)| Expression::Concat { lhs, rhs })
}

fn compile_if(input: CompileIf) -> Valid<Expression, String> {
  let context = input.context;
  let cond = input.cond;
  let then = input.then;
  let els = input.els;

  compile(context, *cond)
    .map(Box::new)
    .zip(compile(context, *then).map(Box::new))
    .zip(compile(context, *els).map(Box::new))
    .map(|((cond, then), els)| Expression::If { cond, then, els })
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
