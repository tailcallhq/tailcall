use std::marker::PhantomData;

use super::expression;
use super::expression::Operation;
use crate::request_template::RequestTemplate;

#[derive(Clone, Debug)]
pub struct Lambda<A> {
    _output: PhantomData<fn() -> A>,
    pub expression: expression::Expression,
}

impl<A> Lambda<A> {
    fn box_expr(self) -> expression::Expression {
        self.expression
    }

    pub fn new(expression: expression::Expression) -> Self {
        Self {
            _output: PhantomData,
            expression,
        }
    }

    pub fn eq(self, other: Self) -> Lambda<bool> {
        Lambda::new(expression::Expression::EqualTo(
            Box::new(self.box_expr()),
            Box::new(other.expression),
        ))
    }

    pub fn to_unsafe_js(self, script: String) -> Lambda<serde_json::Value> {
        Lambda::new(expression::Expression::Unsafe(Operation::JS(
            Box::new(self.box_expr()),
            script,
        )))
    }

    pub fn to_input_path(self, path: Vec<String>) -> Lambda<serde_json::Value> {
        Lambda::new(expression::Expression::Input(Box::new(self.box_expr()), path))
    }
}

impl Lambda<serde_json::Value> {
    pub fn context() -> Self {
        Lambda::new(expression::Expression::Context(expression::Context::Value))
    }

    pub fn context_field(name: String) -> Self {
        Lambda::new(expression::Expression::Context(expression::Context::Path(vec![name])))
    }

    pub fn context_path(path: Vec<String>) -> Self {
        Lambda::new(expression::Expression::Context(expression::Context::Path(path)))
    }

    pub fn from_request_template(req_template: RequestTemplate) -> Lambda<serde_json::Value> {
        Lambda::new(expression::Expression::Unsafe(Operation::Endpoint(req_template)))
    }
}

impl<A> From<A> for Lambda<A>
where
    serde_json::Value: From<A>,
{
    fn from(value: A) -> Self {
        let json = serde_json::Value::from(value);
        Lambda::new(expression::Expression::Literal(json))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    use crate::endpoint::Endpoint;
    use crate::http::RequestContext;
    use crate::lambda::{EvaluationContext, Lambda};
    use crate::request_template::RequestTemplate;

    impl<B> Lambda<B>
    where
        B: DeserializeOwned,
    {
        async fn eval(self) -> Result<B> {
            let req_ctx = RequestContext::default();
            let ctx = EvaluationContext::new(&req_ctx);
            let result = self.expression.eval(&ctx).await?;
            let json = serde_json::to_value(result)?;
            Ok(serde_json::from_value(json)?)
        }
    }

    #[tokio::test]
    async fn test_equal_to_true() {
        let lambda = Lambda::from(1.0).eq(Lambda::from(1.0));
        let result = lambda.eval().await.unwrap();
        assert!(result)
    }

    #[tokio::test]
    async fn test_equal_to_false() {
        let lambda = Lambda::from(1.0).eq(Lambda::from(2.0));
        let result = lambda.eval().await.unwrap();
        assert!(!result)
    }

    #[tokio::test]
    async fn test_endpoint() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/users");
            then
                .status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "name": "Hans" }));
        });

        let endpoint = RequestTemplate::try_from(Endpoint::new(server.url("/users").to_string())).unwrap();
        let result = Lambda::from_request_template(endpoint).eval().await.unwrap();

        assert_eq!(result.as_object().unwrap().get("name").unwrap(), "Hans")
    }

    #[cfg(feature = "unsafe-js")]
    #[tokio::test]
    async fn test_unsafe_js() {
        let result = Lambda::from(1.0).to_unsafe_js("ctx + 100".to_string()).eval().await;
        let f64 = result.unwrap().as_f64().unwrap();
        assert_eq!(f64, 101.0)
    }
}
