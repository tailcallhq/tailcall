#[cfg(test)]
mod tests {
    use core::str;
    use std::sync::Arc;

    use async_graphql_value::ConstValue;
    use tailcall::core::app_context::AppContext;
    use tailcall::core::blueprint::Blueprint;
    use tailcall::core::config::{Config, ConfigModule};
    use tailcall::core::http::RequestContext;
    use tailcall::core::jit::{ConstValueExecutor, Request};
    use tailcall::core::json::JsonLike;
    use tailcall::core::rest::EndpointSet;
    use tailcall_valid::Validator;

    struct TestExecutor {
        app_ctx: Arc<AppContext>,
        req_ctx: Arc<RequestContext>,
    }

    impl TestExecutor {
        async fn try_new() -> anyhow::Result<Self> {
            let sdl =
                tokio::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER).await?;
            let config = Config::from_sdl(&sdl).to_result()?;
            let blueprint = Blueprint::try_from(&ConfigModule::from(config))?;
            let runtime = tailcall::cli::runtime::init(&blueprint);
            let app_ctx = Arc::new(AppContext::new(blueprint, runtime, EndpointSet::default()));
            let req_ctx = Arc::new(RequestContext::from(app_ctx.as_ref()));

            Ok(Self { app_ctx, req_ctx })
        }

        async fn run(&self, request: Request<ConstValue>) -> anyhow::Result<serde_json::Value> {
            let executor = ConstValueExecutor::try_new(&request, &self.app_ctx)?;

            let resp = executor
                .execute(&self.app_ctx, &self.req_ctx, request)
                .await;

            let resp = Arc::into_inner(resp.body).unwrap();

            let resp = str::from_utf8(&resp)?;

            Ok(serde_json::from_str(resp)?)
        }
    }

    #[tokio::test]
    async fn test_executor() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {id title}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_nested() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {title userId user {id name blog} }}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_nested_list() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new(
            "query {posts { id user { id albums { id photos { id title combinedId } } } }}",
        );
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_fragments() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new(
            r#"
            fragment UserPII on User {
              name
              email
              phone
            }

            query {
              users {
                id
                ...UserPII
                username
              }
            }
        "#,
        );
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_fragments_nested() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new(
            r#"
            fragment UserPII on User {
              name
              email
              phone
            }

            query {
              posts {
                id
                user {
                    id
                    ...UserPII
                    username
                }
              }
            }
        "#,
        );
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_arguments() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {user(id: 1) {id}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_arguments_default_value() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {post {id title}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_executor_variables() {
        //  NOTE: This test makes a real HTTP call
        let query = r#"
            query user($id: Int!) {
              user(id: $id) {
                id
                name
              }
            }
        "#;
        let request = Request::new(query);
        let executor = TestExecutor::try_new().await.unwrap();

        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(1))]);
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_operation_plan_cache() {
        fn get_id_value(data: serde_json::Value) -> Option<i64> {
            data.get_key("data")
                .and_then(|v| v.get_key("user"))
                .and_then(|v| v.get_key("id"))
                .and_then(|u| u.as_i64())
        }

        //  NOTE: This test makes a real HTTP call
        let query = r#"
            query user($id: Int!) {
              user(id: $id) {
                id
                name
              }
            }
        "#;
        let request = Request::new(query);
        let executor = TestExecutor::try_new().await.unwrap();

        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(1))]);
        let response = executor.run(request).await.unwrap();

        assert_eq!(get_id_value(response).unwrap(), 1);

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(2))]);
        let response = executor.run(request).await.unwrap();

        assert_eq!(get_id_value(response).unwrap(), 2);
    }

    #[tokio::test]
    async fn test_query_alias() {
        //  NOTE: This test makes a real HTTP call
        let request =
            Request::new("query {user1: user(id: 1) {id name} user2: user(id: 2) {id name}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }
    #[tokio::test]
    async fn test_skip() {
        //  NOTE: This test makes a real HTTP call
        let mut request = Request::new(
            r#"
                query ($TRUE: Boolean!){
                  users {
                    id @skip(if: true)
                    name @skip(if: $TRUE)
                    email @include(if: $TRUE)
                    username @include(if: false)
                    phone @skip(if: false) @include(if: true)
                  }
                }
        "#,
        );
        request
            .variables
            .insert("TRUE".to_string(), ConstValue::Boolean(true));

        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();

        insta::assert_json_snapshot!(response);
    }
}
