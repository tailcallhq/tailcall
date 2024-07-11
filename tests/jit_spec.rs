#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql_value::ConstValue;
    use tailcall::core::app_context::AppContext;
    use tailcall::core::blueprint::Blueprint;
    use tailcall::core::config::{Config, ConfigModule};
    use tailcall::core::jit::{ConstValueExecutor, Request, Variables};
    use tailcall::core::rest::EndpointSet;
    use tailcall::core::valid::Validator;

    async fn new_executor(request: &Request<ConstValue>) -> anyhow::Result<ConstValueExecutor> {
        let sdl = tokio::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER).await?;
        let config = Config::from_sdl(&sdl).to_result()?;
        let blueprint = Blueprint::try_from(&ConfigModule::from(config))?;
        let runtime = tailcall::cli::runtime::init(&blueprint);
        let app_ctx = Arc::new(AppContext::new(blueprint, runtime, EndpointSet::default()));

        Ok(ConstValueExecutor::new(request, app_ctx)?)
    }

    #[tokio::test]
    async fn test_executor() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {id title}}");
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_nested() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {posts {title userId user {id name blog} }}");
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_nested_list() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new(
            "query {posts { id user { id albums { id photos { id title combinedId } } } }}",
        );
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
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
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
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
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_arguments() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {user(id: 1) {id}}");
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_executor_arguments_default_value() {
        //  NOTE: This test makes a real HTTP call
        let request = Request::new("query {post {id title}}");
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
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

        match new_executor(&request).await {
            Ok(_) => panic!("Should fail with unresolved variable"),
            Err(err) => assert_eq!(
                err.to_string(),
                "ResolveInputError: Variable `id` is not defined"
            ),
        };

        let request = request.variables(Variables::from_iter([("id".into(), ConstValue::from(1))]));
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }

    #[tokio::test]
    async fn test_query_alias() {
        //  NOTE: This test makes a real HTTP call
        let request =
            Request::new("query {user1: user(id: 1) {id name} user2: user(id: 2) {id name}}");
        let executor = new_executor(&request).await.unwrap();
        let response = executor.execute(request).await;
        let data = response.data;

        insta::assert_json_snapshot!(data);
    }
}
