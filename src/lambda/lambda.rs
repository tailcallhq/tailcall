use std::marker::PhantomData;

use super::expression::{Context, Expression};
use super::{expression, IO};
use crate::{graphql, grpc, http};

#[derive(Clone)]
pub struct Lambda<A> {
    _output: PhantomData<fn() -> A>,
    pub expression: Expression,
}

impl<A> Lambda<A> {
    fn box_expr(self) -> Box<Expression> {
        Box::new(self.expression)
    }
    pub fn new(expression: Expression) -> Self {
        Self { _output: PhantomData, expression }
    }

    pub fn eq(self, other: Self) -> Lambda<bool> {
        Lambda::new(Expression::EqualTo(
            self.box_expr(),
            Box::new(other.expression),
        ))
    }

    pub fn to_js(self, script: String) -> Lambda<serde_json::Value> {
        Lambda::new(Expression::IO(IO::JS(self.box_expr(), script)))
    }

    pub fn to_input_path(self, path: Vec<String>) -> Lambda<serde_json::Value> {
        Lambda::new(Expression::Input(self.box_expr(), path))
    }
}

impl Lambda<serde_json::Value> {
    pub fn context() -> Self {
        Lambda::new(Expression::Context(expression::Context::Value))
    }

    pub fn context_field(name: String) -> Self {
        Lambda::new(Expression::Context(Context::Path(vec![name])))
    }

    pub fn context_path(path: Vec<String>) -> Self {
        Lambda::new(Expression::Context(Context::Path(path)))
    }

    pub fn from_request_template(req_template: http::RequestTemplate) -> Lambda<serde_json::Value> {
        Lambda::new(Expression::IO(IO::Http {
            req_template,
            group_by: None,
            dl_id: None,
        }))
    }

    pub fn from_graphql_request_template(
        req_template: graphql::RequestTemplate,
        field_name: String,
        batch: bool,
    ) -> Lambda<serde_json::Value> {
        Lambda::new(Expression::IO(IO::GraphQLEndpoint {
            req_template,
            field_name,
            batch,
            dl_id: None,
        }))
    }

    pub fn from_grpc_request_template(
        req_template: grpc::RequestTemplate,
    ) -> Lambda<serde_json::Value> {
        Lambda::new(Expression::IO(IO::Grpc {
            req_template,
            group_by: None,
            dl_id: None,
        }))
    }
}

impl<A> From<A> for Lambda<A>
where
    serde_json::Value: From<A>,
{
    fn from(value: A) -> Self {
        let json = serde_json::Value::from(value);
        Lambda::new(Expression::Literal(json))
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
    use crate::http::{RequestContext, RequestTemplate};
    use crate::lambda::{Concurrent, EmptyResolverContext, Eval, EvaluationContext, Lambda};

    impl<B> Lambda<B>
    where
        B: DeserializeOwned,
    {
        async fn eval(self) -> Result<B> {
            let req_ctx = RequestContext::default();
            let ctx = EvaluationContext::new(&req_ctx, &EmptyResolverContext);
            let result = self.expression.eval(&ctx, &Concurrent::Sequential).await?;
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
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "name": "Hans" }));
        });

        let endpoint =
            RequestTemplate::try_from(Endpoint::new(server.url("/users").to_string())).unwrap();
        let result = Lambda::from_request_template(endpoint)
            .eval()
            .await
            .unwrap();

        assert_eq!(result.as_object().unwrap().get("name").unwrap(), "Hans")
    }

    #[cfg(feature = "unsafe-js")]
    #[tokio::test]
    async fn test_js() {
        let result = Lambda::from(1.0)
            .to_js("ctx + 100".to_string())
            .eval()
            .await;
        let f64 = result.unwrap().as_f64().unwrap();
        assert_eq!(f64, 101.0)
    }
}
