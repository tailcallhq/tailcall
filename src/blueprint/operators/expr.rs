use crate::blueprint::*;
use crate::config;
use crate::config::{ExprBody, Field, If};
use crate::lambda::{Expression, List, Logic, Math, Relation};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

struct CompilationContext<'a> {
    config_field: &'a config::Field,
    operation_type: &'a config::GraphQLOperationType,
    config_module: &'a config::ConfigModule,
}

pub fn update_expr(
    operation_type: &config::GraphQLOperationType,
) -> TryFold<'_, (&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config_module, field, _, _), b_field| {
            let Some(expr) = &field.expr else {
                return Valid::succeed(b_field);
            };

            let context = CompilationContext { config_module, operation_type, config_field: field };

            compile(&context, expr.body.clone()).map(|compiled| b_field.resolver(Some(compiled)))
        },
    )
}

///
/// Compiles a list of Exprs into a list of Expressions
///
fn compile_list(
    context: &CompilationContext,
    expr_vec: Vec<ExprBody>,
) -> Valid<Vec<Expression>, String> {
    Valid::from_iter(expr_vec, |value| compile(context, value))
}

///
/// Compiles a tuple of Exprs into a tuple of Expressions
///
fn compile_ab(
    context: &CompilationContext,
    ab: (ExprBody, ExprBody),
) -> Valid<(Expression, Expression), String> {
    compile(context, ab.0).zip(compile(context, ab.1))
}

///
/// Compiles expr into Expression
///
fn compile(ctx: &CompilationContext, expr: ExprBody) -> Valid<Expression, String> {
    let config_module = ctx.config_module;
    let field = ctx.config_field;
    let operation_type = ctx.operation_type;
    match expr {
        // Io Expr
        ExprBody::Http(http) => compile_http(config_module, field, &http),
        ExprBody::Grpc(grpc) => {
            let grpc = CompileGrpc {
                config_module,
                field,
                operation_type,
                grpc: &grpc,
                validate_with_schema: false,
            };
            compile_grpc(grpc)
        }
        ExprBody::GraphQL(gql) => compile_graphql(config_module, operation_type, &gql),

        // Safe Expr
        ExprBody::Const(value) => Valid::from(
            DynamicValue::try_from(&value).map_err(|e| ValidationError::new(e.to_string())),
        )
        .and_then(|value| {
            compile_const(CompileConst { config_module, field, value: &value, validate: false })
        }),

        // Logic
        ExprBody::If(If { ref cond, on_true: ref then, on_false: ref els }) => {
            compile(ctx, *cond.clone())
                .map(Box::new)
                .fuse(compile(ctx, *then.clone()).map(Box::new))
                .fuse(compile(ctx, *els.clone()).map(Box::new))
                .map(|(cond, then, els)| {
                    Expression::Logic(Logic::If { cond, then, els }).parallel_when(expr.has_io())
                })
        }

        ExprBody::And(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Logic(Logic::And(a)).parallel_when(expr.has_io())),
        ExprBody::Or(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Logic(Logic::Or(a)).parallel_when(expr.has_io())),
        ExprBody::Cond(default, list) => Valid::from_iter(list, |(cond, operation)| {
            compile_ab(ctx, (*cond, *operation))
                .map(|(cond, operation)| (Box::new(cond), Box::new(operation)))
        })
        .and_then(|mut list| {
            compile(ctx, *default).map(|default| {
                list.push((
                    Box::new(Expression::Literal(DynamicValue::Value(true.into()))),
                    Box::new(default),
                ));
                Expression::Logic(Logic::Cond(list))
            })
        }),
        ExprBody::DefaultTo(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Logic(Logic::DefaultTo(Box::new(a), Box::new(b)))),
        ExprBody::IsEmpty(a) => {
            compile(ctx, *a).map(|a| Expression::Logic(Logic::IsEmpty(Box::new(a))))
        }
        ExprBody::Not(a) => compile(ctx, *a).map(|a| Expression::Logic(Logic::Not(Box::new(a)))),

        // List
        ExprBody::Concat(ref values) => compile_list(ctx, values.clone())
            .map(|a| Expression::List(List::Concat(a)).parallel_when(expr.has_io())),

        // Relation
        ExprBody::Intersection(ref values) => compile_list(ctx, values.clone())
            .map(|a| Expression::Relation(Relation::Intersection(a)).parallel_when(expr.has_io())),
        ExprBody::Difference(a, b) => compile_list(ctx, a)
            .zip(compile_list(ctx, b))
            .map(|(a, b)| Expression::Relation(Relation::Difference(a, b))),
        ExprBody::Equals(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::Equals(Box::new(a), Box::new(b)))),
        ExprBody::Gt(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::Gt(Box::new(a), Box::new(b)))),
        ExprBody::Gte(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::Gte(Box::new(a), Box::new(b)))),
        ExprBody::Lt(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::Lt(Box::new(a), Box::new(b)))),
        ExprBody::Lte(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::Lte(Box::new(a), Box::new(b)))),
        ExprBody::Max(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Relation(Relation::Max(a)).parallel_when(expr.has_io())),
        ExprBody::Min(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Relation(Relation::Min(a)).parallel_when(expr.has_io())),
        ExprBody::PathEq(a, path, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::PathEq(Box::new(a), path, Box::new(b)))),
        ExprBody::PropEq(a, path, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Relation(Relation::PropEq(Box::new(a), path, Box::new(b)))),
        ExprBody::SortPath(a, path) => compile(ctx, *a)
            .map(|a| Expression::Relation(Relation::SortPath(Box::new(a), path.clone()))),
        ExprBody::SymmetricDifference(a, b) => compile_list(ctx, a)
            .zip(compile_list(ctx, b))
            .map(|(a, b)| Expression::Relation(Relation::SymmetricDifference(a, b))),
        ExprBody::Union(a, b) => compile_list(ctx, a)
            .zip(compile_list(ctx, b))
            .map(|(a, b)| Expression::Relation(Relation::Union(a, b))),

        // Math
        ExprBody::Mod(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Math(Math::Mod(Box::new(a), Box::new(b)))),
        ExprBody::Add(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Math(Math::Add(Box::new(a), Box::new(b)))),
        ExprBody::Dec(a) => compile(ctx, *a).map(|a| Expression::Math(Math::Dec(Box::new(a)))),
        ExprBody::Divide(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Math(Math::Divide(Box::new(a), Box::new(b)))),
        ExprBody::Inc(a) => compile(ctx, *a).map(|a| Expression::Math(Math::Inc(Box::new(a)))),
        ExprBody::Multiply(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Math(Math::Multiply(Box::new(a), Box::new(b)))),
        ExprBody::Negate(a) => {
            compile(ctx, *a).map(|a| Expression::Math(Math::Negate(Box::new(a))))
        }
        ExprBody::Product(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Math(Math::Product(a)).parallel_when(expr.has_io())),
        ExprBody::Subtract(a, b) => compile_ab(ctx, (*a, *b))
            .map(|(a, b)| Expression::Math(Math::Subtract(Box::new(a), Box::new(b)))),
        ExprBody::Sum(ref list) => compile_list(ctx, list.clone())
            .map(|a| Expression::Math(Math::Sum(a)).parallel_when(expr.has_io())),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    use pretty_assertions::assert_eq;
    use serde_json::{json, Number};

    use super::{compile, CompilationContext};
    use crate::config::{ConfigModule, Expr, Field, GraphQLOperationType};
    use crate::http::RequestContext;
    use crate::lambda::{Concurrent, Eval, EvaluationContext, ResolverContextLike};
    use crate::valid::Validator;

    #[derive(Default)]
    struct Context<'a> {
        value: Option<&'a async_graphql_value::ConstValue>,
        args: Option<
            &'a indexmap::IndexMap<async_graphql_value::Name, async_graphql_value::ConstValue>,
        >,
        field: Option<async_graphql::SelectionField<'a>>,
        errors: Arc<Mutex<Vec<async_graphql::ServerError>>>,
    }

    impl<'a> ResolverContextLike<'a> for Context<'a> {
        fn value(&'a self) -> Option<&'a async_graphql_value::ConstValue> {
            self.value
        }

        fn args(
            &'a self,
        ) -> Option<
            &'a indexmap::IndexMap<async_graphql_value::Name, async_graphql_value::ConstValue>,
        > {
            self.args
        }

        fn field(&'a self) -> Option<async_graphql::SelectionField> {
            self.field
        }

        fn add_error(&'a self, error: async_graphql::ServerError) {
            self.errors.lock().unwrap().push(error);
        }
    }

    impl Expr {
        async fn eval(expr: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            let expr = serde_json::from_value::<Expr>(expr)?;
            let config_module = ConfigModule::default();
            let field = Field::default();
            let operation_type = GraphQLOperationType::Query;
            let context = CompilationContext {
                config_module: &config_module,
                config_field: &field,
                operation_type: &operation_type,
            };
            let expression = compile(&context, expr.body.clone()).to_result()?;
            let req_ctx = RequestContext::default();
            let graphql_ctx = Context::default();
            let ctx = EvaluationContext::new(&req_ctx, &graphql_ctx);
            let value = expression.eval(&ctx, &Concurrent::default()).await?;

            Ok(serde_json::to_value(value)?)
        }
    }

    #[tokio::test]
    async fn test_is_truthy() {
        let actual = Expr::eval(json!({"body": {"inc": {"const": 1}}}))
            .await
            .unwrap();
        let expected = json!(2.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_add() {
        let actual = Expr::eval(json!({"body": {"add": [{"const": 40}, {"const": 2}]}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_subtract() {
        let actual = Expr::eval(json!({"body": {"subtract": [{"const": 52}, {"const": 10}]}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_multiply() {
        let actual = Expr::eval(json!({"body": {"multiply": [{"const": 7}, {"const": 6}]}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_mod() {
        let actual = Expr::eval(json!({"body": {"mod": [{"const": 1379}, {"const": 1337}]}}))
            .await
            .unwrap();
        let expected = json!(42);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_div1() {
        let actual = Expr::eval(json!({"body": {"divide": [{"const": 9828}, {"const": 234}]}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_div2() {
        let actual = Expr::eval(json!({"body": {"divide": [{"const": 105}, {"const": 2.5}]}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_inc() {
        let actual = Expr::eval(json!({"body": {"inc": {"const": 41}}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_dec() {
        let actual = Expr::eval(json!({"body": {"dec": {"const": 43}}}))
            .await
            .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_product() {
        let actual =
            Expr::eval(json!({"body": {"product": [{"const": 7}, {"const": 3}, {"const": 2}]}}))
                .await
                .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_math_sum() {
        let actual =
            Expr::eval(json!({"body": {"sum": [{"const": 20}, {"const": 15}, {"const": 7}]}}))
                .await
                .unwrap();
        let expected = json!(42.0);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_and_true() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"and": [{"const": true}, {"const": true}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(
            json!({"body": {"and": [{"const": true}, {"const": true}, {"const": true}]}}),
        )
        .await
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_and_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"and": [{"const": true}, {"const": false}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(
            json!({"body": {"and": [{"const": true}, {"const": true}, {"const": false}]}}),
        )
        .await
        .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"and": [{"const": false}, {"const": false}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_is_empty_true() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": []}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": {}}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": ""}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": null}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_is_empty_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": [1]}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": {"a": 1}}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": "a"}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": 1}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"isEmpty": {"const": false}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_not_true() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"not": {"const": true}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"not": {"const": 1}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_not_false() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"not": {"const": false}}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_or_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"or": [{"const": false}, {"const": false}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(
            json!({"body": {"or": [{"const": false}, {"const": false}, {"const": false}]}}),
        )
        .await
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_or_true() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"or": [{"const": true}, {"const": false}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(
            json!({"body": {"or": [{"const": false}, {"const": false}, {"const": true}]}}),
        )
        .await
        .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"or": [{"const": true}, {"const": true}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_cond() {
        let expected = json!(0);

        let actual = Expr::eval(
            json!({"body": {"cond": [{"const": 0}, [[{"const": false}, {"const": 1}], [{"const": false}, {"const": 2}]]]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!(1);

        let actual = Expr::eval(
            json!({"body": {"cond": [{"const": 0}, [[{"const": true}, {"const": 1}], [{"const": true}, {"const": 2}]]]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!(2);
        let actual = Expr::eval(
            json!({"body": {"cond": [{"const": 0}, [[{"const": false}, {"const": 1}], [{"const": true}, {"const": 2}]]]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_logic_default_to() {
        let expected = json!(0);
        let actual = Expr::eval(json!({"body": {"defaultTo": [{"const": null}, {"const": 0}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!(true);
        let actual = Expr::eval(json!({"body": {"defaultTo": [{"const": ""}, {"const": true}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_concat() {
        let expected = json!([1, 2, 3, 4]);
        let actual =
            Expr::eval(json!({"body": {"concat": [{"const": [1, 2]}, {"const": [3, 4]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_intersection() {
        let expected = json!([3]);
        let actual = Expr::eval(
            json!({"body": {"intersection": [{"const": [1, 2, 3]}, {"const": [3, 4, 5]}]}}),
        )
        .await
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_difference() {
        let expected = json!([1]);
        let actual = Expr::eval(
            json!({"body": {"difference": [[{"const": 1}, {"const": 2}, {"const": 3}], [{"const": 2}, {"const": 3}]]}}),
        )
            .await
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_simmetric_difference() {
        let expected = json!([1]);

        let actual = Expr::eval(
            json!({"body": {"symmetricDifference": [[{"const": 1}, {"const": 2}, {"const": 3}], [{"const": 2}, {"const": 3}]]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(
            json!({"body": {"symmetricDifference": [[{"const": 2}, {"const": 3}], [{"const": 1}, {"const": 2}, {"const": 3}]]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_union() {
        let expected = serde_json::from_value::<HashSet<Number>>(json!([1, 2, 3, 4])).unwrap();

        let actual = Expr::eval(json!({"body": {"union": [[{"const": 1}, {"const": 2}, {"const": 3}], [{"const": 2}, {"const": 3}, {"const": 4}]]}}))
            .await
            .unwrap();
        let actual = serde_json::from_value::<HashSet<Number>>(actual).unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_eq_true() {
        let expected = json!(true);

        let actual =
            Expr::eval(json!({"body": {"eq": [{"const": [1, 2, 3]}, {"const": [1, 2, 3]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"eq": [{"const": "abc"}, {"const": "abc"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"eq": [{"const": true}, {"const": true}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_eq_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"eq": [{"const": [1, 2, 3]}, {"const": [1, 2]}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"eq": [{"const": "abc"}, {"const": 1}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"eq": [{"const": "abc"}, {"const": "ac"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_gt_true() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": [1, 2, 3]}, {"const": [1, 2]}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": "bc"}, {"const": "ab"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": 4}, {"const": -1}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_gt_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": [1, 2, 3]}, {"const": [2, 2]}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": "abc"}, {"const": "z"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gt": [{"const": 0}, {"const": 3.74}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_lt_true() {
        let expected = json!(true);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": [1, 2, 3]}, {"const": [2, 2]}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": "abc"}, {"const": "z"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": 0}, {"const": 3.74}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_lt_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": [1, 2, 3]}, {"const": [1, 2]}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": "bc"}, {"const": "ab"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lt": [{"const": 4}, {"const": -1}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_gte_true() {
        let expected = json!(true);

        let actual =
            Expr::eval(json!({"body": {"gte": [{"const": [1, 2, 3]}, {"const": [1, 2]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gte": [{"const": "bc"}, {"const": "ab"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gte": [{"const": 4}, {"const": -1}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gte": [{"const": 4}, {"const": 4}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_gte_false() {
        let expected = json!(false);

        let actual =
            Expr::eval(json!({"body": {"gte": [{"const": [1, 2, 3]}, {"const": [2, 2]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gte": [{"const": "abc"}, {"const": "z"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"gte": [{"const": 0}, {"const": 3.74}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_lte_true() {
        let expected = json!(true);

        let actual =
            Expr::eval(json!({"body": {"lte": [{"const": [1, 2, 3]}, {"const": [1, 2, 3]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lte": [{"const": 4}, {"const": 4}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual =
            Expr::eval(json!({"body": {"lte": [{"const": [1, 2, 3]}, {"const": [2, 2]}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lte": [{"const": "abc"}, {"const": "z"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual = Expr::eval(json!({"body": {"lte": [{"const": 0}, {"const": 3.74}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_lte_false() {
        let expected = json!(false);

        let actual = Expr::eval(json!({"body": {"lte": [{"const": "bc"}, {"const": "ab"}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_max() {
        let expected = json!(923.83);
        let actual = Expr::eval(
            json!({"body": {"max": [{"const": 1}, {"const": 23}, {"const": -423}, {"const": 0}, {"const": 923.83}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!("z");
        let actual = Expr::eval(
            json!({"body": {"max": [{"const": "abc"}, {"const": "z"}, {"const": "bcd"}, {"const": "foo"}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!([2, 3]);
        let actual = Expr::eval(
            json!({"body": {"max": [{"const": [2, 3]}, {"const": [0, 1, 2]}, {"const": [-1, 0, 0, 0]}, {"const": [1]}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_min() {
        let expected = json!(-423);
        let actual = Expr::eval(
            json!({"body": {"min": [{"const": 1}, {"const": 23}, {"const": -423}, {"const": 0}, {"const": 923.83}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!("abc");
        let actual = Expr::eval(
            json!({"body": {"min": [{"const": "abc"}, {"const": "z"}, {"const": "bcd"}, {"const": "foo"}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let expected = json!([-1, 0, 0, 0]);
        let actual = Expr::eval(
            json!({"body": {"min": [{"const": [2, 3]}, {"const": [0, 1, 2]}, {"const": [-1, 0, 0, 0]}, {"const": [1]}]}}),
        )
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_sort_path() {
        let expected = json!([2, 3, 4]);
        let actual = Expr::eval(json!({"body": {"sortPath": [{"const": [4, 2, 3]}, []]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_path_eq_true() {
        let expected = json!(true);
        let actual = Expr::eval(json!({"body": {"pathEq": [{"const": 10}, [], {"const": 10}]}}))
            .await
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_relation_path_eq_false() {
        let expected = json!(false);
        let actual =
            Expr::eval(json!({"body": {"pathEq": [{"const": "ab"}, [], {"const": "bcd"}]}}))
                .await
                .unwrap();
        assert_eq!(actual, expected);
    }

    // TODO: add tests for all other expr operators
}
