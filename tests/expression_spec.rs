#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tailcall::core::blueprint::{Blueprint, DynamicValue};
    use tailcall::core::http::RequestContext;
    use tailcall::core::ir::{EmptyResolverContext, Eval, EvaluationContext, EvaluationError, IR};
    use tailcall::core::mustache::Mustache;

    async fn eval(expr: &IR) -> Result<Value, EvaluationError> {
        let runtime = tailcall::cli::runtime::init(&Blueprint::default());
        let req_ctx = RequestContext::new(runtime);
        let res_ctx = EmptyResolverContext {};
        let eval_ctx = EvaluationContext::new(&req_ctx, &res_ctx);
        expr.eval(eval_ctx).await
    }

    #[tokio::test]
    async fn test_and_then() {
        let abcde = DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap();
        let expr = IR::Dynamic(abcde)
            .and_then(IR::Dynamic(DynamicValue::Mustache(
                Mustache::parse("{{args.a}}").unwrap(),
            )))
            .and_then(IR::Dynamic(DynamicValue::Mustache(
                Mustache::parse("{{args.b}}").unwrap(),
            )))
            .and_then(IR::Dynamic(DynamicValue::Mustache(
                Mustache::parse("{{args.c}}").unwrap(),
            )))
            .and_then(IR::Dynamic(DynamicValue::Mustache(
                Mustache::parse("{{args.d}}").unwrap(),
            )));

        let actual = eval(&expr).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_with_args() {
        let args =
            IR::Dynamic(DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap());

        let expr = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.a.b.c.d}}").unwrap(),
        ))
        .with_args(args);

        let actual = eval(&expr).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_with_args_piping() {
        let args =
            IR::Dynamic(DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap());

        let expr = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.a}}").unwrap(),
        ))
        .with_args(args)
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.b}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.c}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.d}}").unwrap(),
        )));

        let actual = eval(&expr).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_optional_dot_in_expression() {
        let args =
            IR::Dynamic(DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap());

        let expr_with_dot = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.a.b.c.d}}").unwrap(),
        ))
        .with_args(args.clone());

        let expr_without_dot = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.a.b.c.d}}").unwrap(),
        ))
        .with_args(args);

        let actual_with_dot = eval(&expr_with_dot).await.unwrap();
        let actual_without_dot = eval(&expr_without_dot).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual_with_dot, expected);
        assert_eq!(actual_without_dot, expected);
    }

    #[tokio::test]
    async fn test_optional_dot_piping() {
        let args =
            IR::Dynamic(DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap());

        let expr = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.a}}").unwrap(),
        ))
        .with_args(args)
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.b}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.c}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.d}}").unwrap(),
        )));

        let actual = eval(&expr).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_mixed_dot_usages() {
        let args =
            IR::Dynamic(DynamicValue::try_from(&json!({"a": {"b": {"c": {"d": "e"}}}})).unwrap());

        let expr = IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.a}}").unwrap(),
        ))
        .with_args(args)
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.b}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{.args.c}}").unwrap(),
        )))
        .and_then(IR::Dynamic(DynamicValue::Mustache(
            Mustache::parse("{{args.d}}").unwrap(),
        )));

        let actual = eval(&expr).await.unwrap();
        let expected = Value::from_json(json!("e")).unwrap();

        assert_eq!(actual, expected);
    }
}
