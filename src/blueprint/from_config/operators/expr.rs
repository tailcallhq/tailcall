use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, Field};
use crate::lambda::{Expression, List, Logic, Relation};
use crate::try_fold::TryFold;
use crate::valid::Valid;

struct CompilationContext<'a> {
  config_field: &'a config::Field,
  operation_type: &'a config::GraphQLOperationType,
  config: &'a config::Config,
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

///
/// Compiles a list of Exprs into a list of Expressions
///
fn compile_list(context: &CompilationContext, expr_vec: Vec<ExprBody>) -> Valid<Vec<Expression>, String> {
  Valid::from_iter(expr_vec, |value| compile(context, value))
}

///
/// Compiles expr into Expression
///
fn compile(ctx: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
  let config = ctx.config;
  let field = ctx.config_field;
  let operation_type = ctx.operation_type;

  match expr {
    // Unsafe Expr
    ExprBody::Http(http) => compile_http(config, field, &http),
    ExprBody::Grpc(grpc) => {
      let grpc = CompileGrpc { config, field, operation_type, grpc: &grpc, validate_with_schema: false };
      compile_grpc(grpc)
    }
    ExprBody::GraphQL(gql) => compile_graphql(config, operation_type, &gql),

    // Safe Expr
    ExprBody::Const(value) => compile_const(CompileConst { config, field, value: &value, validate: false }),
    ExprBody::If { cond, on_true: then, on_false: els } => compile(ctx, *cond)
      .map(Box::new)
      .zip(compile(ctx, *then).map(Box::new))
      .zip(compile(ctx, *els).map(Box::new))
      .map(|((cond, then), els)| Expression::Logic(Logic::If { cond, then, els })),
    ExprBody::Concat(values) => compile_list(ctx, values).map(|a| Expression::List(List::Concat(a))),
    ExprBody::Intersection(values) => {
      compile_list(ctx, values).map(|a| Expression::Relation(Relation::Intersection(a)))
    }
  }
}
