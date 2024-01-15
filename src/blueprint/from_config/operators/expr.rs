use crate::blueprint::*;
use crate::config;
use crate::config::{Config, ExprBody, Field};
use crate::lambda::{Expression, List, Logic, Math, Relation};
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
/// Compiles a tuple of Exprs into a tuple of Expressions
///
fn compile_ab(context: &CompilationContext, ab: (ExprBody, ExprBody)) -> Valid<(Expression, Expression), String> {
  compile(context, ab.0).zip(compile(context, ab.1))
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

    // Logic
    ExprBody::If { cond, on_true: then, on_false: els } => compile(ctx, *cond)
      .map(Box::new)
      .zip(compile(ctx, *then).map(Box::new))
      .zip(compile(ctx, *els).map(Box::new))
      .map(|((cond, then), els)| Expression::Logic(Logic::If { cond, then, els })),

    ExprBody::AllPass(list) => compile_list(ctx, list).map(|a| Expression::Logic(Logic::AllPass(a))),
    ExprBody::And(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Logic(Logic::And(Box::new(a), Box::new(b))))
    }
    ExprBody::AnyPass(list) => compile_list(ctx, list).map(|a| Expression::Logic(Logic::AnyPass(a))),
    ExprBody::Cond(list) => Valid::from_iter(list, |(cond, operation)| {
      compile_ab(ctx, (*cond, *operation)).map(|(cond, operation)| (Box::new(cond), Box::new(operation)))
    })
    .map(|list| Expression::Logic(Logic::Cond(list))),
    ExprBody::DefaultTo(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Logic(Logic::DefaultTo(Box::new(a), Box::new(b))))
    }
    ExprBody::IsEmpty(a) => compile(ctx, *a).map(|a| Expression::Logic(Logic::IsEmpty(Box::new(a)))),
    ExprBody::Not(a) => compile(ctx, *a).map(|a| Expression::Logic(Logic::Not(Box::new(a)))),
    ExprBody::Or(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Logic(Logic::Or(Box::new(a), Box::new(b))))
    }

    // List
    ExprBody::Concat(values) => compile_list(ctx, values).map(|a| Expression::List(List::Concat(a))),

    // Relation
    ExprBody::Intersection(values) => {
      compile_list(ctx, values).map(|a| Expression::Relation(Relation::Intersection(a)))
    }

    // Math
    ExprBody::Mod(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Math(Math::Mod(Box::new(a), Box::new(b))))
    }
    ExprBody::Add(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Math(Math::Add(Box::new(a), Box::new(b))))
    }
    ExprBody::Dec(a) => compile(ctx, *a).map(|a| Expression::Math(Math::Dec(Box::new(a)))),
    ExprBody::Divide(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Math(Math::Divide(Box::new(a), Box::new(b))))
    }
    ExprBody::Inc(a) => compile(ctx, *a).map(|a| Expression::Math(Math::Inc(Box::new(a)))),
    ExprBody::Multiply(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Math(Math::Multiply(Box::new(a), Box::new(b))))
    }
    ExprBody::Negate(a) => compile(ctx, *a).map(|a| Expression::Math(Math::Negate(Box::new(a)))),
    ExprBody::Product(list) => compile_list(ctx, list).map(|a| Expression::Math(Math::Product(a))),
    ExprBody::Subtract(a, b) => {
      compile_ab(ctx, (*a, *b)).map(|(a, b)| Expression::Math(Math::Subtract(Box::new(a), Box::new(b))))
    }
    ExprBody::Sum(list) => compile_list(ctx, list).map(|a| Expression::Math(Math::Sum(a))),
  }
}
