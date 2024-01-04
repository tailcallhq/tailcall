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

fn is_effect(expr: &ExprBody) -> bool {
  match expr {
    ExprBody::Http(_) | ExprBody::Grpc(_) | ExprBody::Const(_) | ExprBody::GraphQL(_) => true,
    ExprBody::If { .. } => false,
  }
}

fn compile_effect(context: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  let operation_type = context.operation_type;
  let config = context.config;
  let config_field = context.config_field;

  match expr {
    ExprBody::Http(http) => compile_http(config, config_field, &http).trace("http"),
    ExprBody::Const(const_field) => compile_const(CompileConst {
      config,
      field: config_field,
      const_field: &const_field,
      validate_with_schema: false,
    })
    .trace("const"),
    ExprBody::GraphQL(graphql) => compile_graphql(config, operation_type, &graphql).trace("graphql"),
    ExprBody::Grpc(grpc) => compile_grpc(CompileGrpc {
      config,
      operation_type,
      field: config_field,
      grpc: &grpc,
      validate_with_schema: false,
    })
    .trace("grpc"),
    _ => Valid::fail(format!(
      "expected one of http, const, grpc or graphql. Found {:?}",
      expr
    )),
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
    .trace("condition")
    .map(Box::new)
    .zip(compile(context, *then).trace("then").map(Box::new))
    .zip(compile(context, *els).trace("else").map(Box::new))
    .map(|((condition, then), els)| Expression::If { condition, then, els })
}

fn compile(context: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  if is_effect(&expr) {
    compile_effect(context, expr)
  } else {
    match expr {
      ExprBody::If { cond: condition, then, els } => {
        compile_if(CompileIf { context, condition, then, els }).trace("if")
      }
      _ => Valid::fail("unsupported expression".to_string()), // unreachable if is_effect is
                                                              // correct
    }
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
