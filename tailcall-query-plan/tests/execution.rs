use std::fs;
use std::path::Path;

use async_graphql::parser::parse_query;
use tailcall::blueprint::Blueprint;
use tailcall::config::{Config, ConfigModule};
use tailcall::http::RequestContext;
use tailcall::valid::Validator;
use tailcall_query_plan::execution::executor::Executor;
use tailcall_query_plan::execution::simple::SimpleExecutionBuilder;
use tailcall_query_plan::plan::{GeneralPlan, OperationPlan};

#[tokio::test]
async fn test_simple() {
    let root_dir = Path::new(tailcall_fixtures::configs::SELF);
    let config = fs::read_to_string(root_dir.join("user-posts.graphql")).unwrap();
    let config = Config::from_sdl(&config).to_result().unwrap();
    let config = ConfigModule::from(config);
    let blueprint = Blueprint::try_from(&config).unwrap();

    let general_plan = GeneralPlan::from_operation(&blueprint.definitions, &blueprint.query());

    insta::assert_snapshot!("general_plan", general_plan);

    let document =
        parse_query(fs::read_to_string(root_dir.join("user-posts-query.graphql")).unwrap())
            .unwrap();

    for (name, operation) in document.operations.iter() {
        let name = name.unwrap().to_string();
        let operation_plan =
            OperationPlan::from_request(&general_plan, &operation.node.selection_set.node);

        insta::assert_snapshot!(format!("{name}_operation_plan"), operation_plan);

        let execution_builder = SimpleExecutionBuilder {};
        let execution_plan = execution_builder.build(&operation_plan);

        insta::assert_snapshot!(format!("{name}_execution_plan"), execution_plan);

        let executor = Executor::new(&general_plan, &operation_plan);

        let runtime = tailcall::cli::runtime::init(&Blueprint::default());
        let req_ctx = RequestContext::new(runtime);
        let execution_result = executor.execute(&req_ctx, &execution_plan).await;

        insta::assert_snapshot!(format!("{name}_execution_result"), execution_result);

        let result = operation_plan.collect_value(execution_result);

        match result {
            Ok(result) => insta::assert_json_snapshot!(format!("{name}_output"), result),
            Err(err) => insta::assert_debug_snapshot!(format!("{name}_output"), err),
        }
    }
}
