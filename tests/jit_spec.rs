#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::Pos;
    use async_graphql_value::ConstValue;
    use tailcall::core::app_context::AppContext;
    use tailcall::core::blueprint::{Blueprint, RuntimeConfig};
    use tailcall::core::config::{Config, ConfigModule};
    use tailcall::core::http::RequestContext;
    use tailcall::core::jit::graphql_error::GraphQLError;
    use tailcall::core::jit::{
        BuildError, ConstValueExecutor, Error, Positioned, Request, ResolveInputError, Response,
    };
    use tailcall::core::json::{JsonLike, JsonObjectLike};
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
            let config_module = ConfigModule::from(config);
            let blueprint = Blueprint::try_from(&config_module)?;
            let runtime_config = RuntimeConfig::try_from(&config_module)?;
            let runtime = tailcall::cli::runtime::init(&runtime_config);
            let app_ctx = Arc::new(AppContext::new(
                blueprint,
                runtime,
                runtime_config,
                EndpointSet::default(),
            ));
            let req_ctx = Arc::new(RequestContext::from(app_ctx.as_ref()));

            Ok(Self { app_ctx, req_ctx })
        }

        async fn run(&self, request: Request<ConstValue>) -> anyhow::Result<Response<ConstValue>> {
            let executor = ConstValueExecutor::try_new(&request, &self.app_ctx)?;

            Ok(executor.execute(&self.req_ctx, &request).await)
        }
    }

    #[tokio::test]
    async fn test_executor() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {id title}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_nested() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {title userId user {id name blog} }}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_nested_list() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new(
            "query {posts { id user { id albums { id photos { id title combinedId } } } }}",
        );
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
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
        let data = response.data;

        insta::assert_json_snapshot!(data);
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
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_arguments() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {user(id: 1) {id}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_arguments_default_value() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {post {id title}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
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

        let resp = executor.run(request).await.unwrap();
        let errs: Vec<GraphQLError> = vec![Positioned::new(
            Error::BuildError(BuildError::ResolveInputError(
                ResolveInputError::VariableIsNotFound("id".to_string()),
            )),
            Pos::default().into(),
        )
        .into()];
        assert_eq!(format!("{:?}", resp.errors), format!("{:?}", errs));

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(1))]);
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_operation_plan_cache() {
        fn get_id_value(data: ConstValue) -> Option<i64> {
            data.as_object()
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

        let resp = executor.run(request).await.unwrap();
        let errs: Vec<GraphQLError> = vec![Positioned::new(
            Error::BuildError(BuildError::ResolveInputError(
                ResolveInputError::VariableIsNotFound("id".to_string()),
            )),
            Pos::default().into(),
        )
        .into()];
        assert_eq!(format!("{:?}", resp.errors), format!("{:?}", errs));

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(1))]);
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        assert_eq!(get_id_value(data).unwrap(), 1);

        let request = Request::new(query);
        let request = request.variables([("id".into(), ConstValue::from(2))]);
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        assert_eq!(get_id_value(data).unwrap(), 2);
    }

    #[tokio::test]
    async fn test_query_alias() {
        //  NOTE: This test makes a real HTTP call
        let request =
            Request::new("query {user1: user(id: 1) {id name} user2: user(id: 2) {id name}}");
        let executor = TestExecutor::try_new().await.unwrap();
        let response = executor.run(request).await.unwrap();
        let data = response.data;

        insta::assert_json_snapshot!(data);
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
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }
}
