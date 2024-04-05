use std::{fmt::Display, sync::Mutex};

use anyhow::{anyhow, Result};
use async_graphql::Value;
use futures_util::future::join_all;
use indexmap::IndexMap;

use crate::{
    http::RequestContext,
    lambda::{EvaluationContext, ResolverContextLike},
    query_plan::{plan::GeneralPlan, resolver::Id},
};

use super::execution::ExecutionStep;

pub struct Executor<'a> {
    general_plan: &'a GeneralPlan,
    resolved: Mutex<IndexMap<Id, Result<Value>>>,
}

impl<'a> Display for Executor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.resolved.lock().unwrap())
    }
}

impl<'a> Executor<'a> {
    pub fn new(general_plan: &'a GeneralPlan) -> Self {
        Self { general_plan, resolved: Mutex::new(IndexMap::default()) }
    }

    #[async_recursion::async_recursion]
    pub async fn execute<Ctx: ResolverContextLike<'a> + Sync + Send>(
        &self,
        req_ctx: &'a RequestContext,
        graphql_ctx: &'a Ctx,
        execution: &ExecutionStep,
    ) {
        match execution {
            ExecutionStep::Resolve(id) => {
                let field_plan = self.general_plan.field_plans.get(**id);

                let result = if let Some(field_plan) = field_plan {
                    let eval_ctx = EvaluationContext::new(req_ctx, graphql_ctx);

                    field_plan.eval(eval_ctx).await
                } else {
                    Err(anyhow!("Failed to resolve field_plan for id: {id}"))
                };

                self.resolved.lock().unwrap().insert(*id, result);
            }
            ExecutionStep::Sequential(steps) => {
                for step in steps {
                    self.execute(req_ctx, graphql_ctx, step).await;
                }
            }
            ExecutionStep::Parallel(steps) => {
                join_all(
                    steps
                        .iter()
                        .map(|step| self.execute(req_ctx, graphql_ctx, step)),
                )
                .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use async_graphql::parser::parse_query;

    use crate::{
        blueprint::Blueprint,
        config::{Config, ConfigModule},
        http::RequestContext,
        lambda::EmptyResolverContext,
        query_plan::{
            execution::{executor::Executor, simple::SimpleExecutionBuilder},
            plan::{GeneralPlan, OperationPlan},
        },
        valid::Validator,
    };

    #[tokio::test]
    async fn test_simple() {
        let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/query_plan/tests");
        let config = fs::read_to_string(root_dir.join("user-posts.graphql")).unwrap();
        let config = Config::from_sdl(&config).to_result().unwrap();
        let config = ConfigModule::from(config);
        let blueprint = Blueprint::try_from(&config).unwrap();

        let general_plan = GeneralPlan::from_operation(&blueprint.definitions, &blueprint.query());

        let document =
            parse_query(fs::read_to_string(root_dir.join("user-posts-query.graphql")).unwrap())
                .unwrap();

        for (name, operation) in document.operations.iter() {
            let name = name.unwrap().to_string();
            let operation_plan =
                OperationPlan::from_request(&general_plan, &operation.node.selection_set.node);
            let execution_builder = SimpleExecutionBuilder {};
            let execution_plan = execution_builder.build(&operation_plan);
            let mut executor = Executor::new(&general_plan);

            let runtime = crate::cli::runtime::init(&Blueprint::default());
            let req_ctx = RequestContext::new(runtime);
            let graphql_ctx = EmptyResolverContext {};
            executor
                .execute(&req_ctx, &graphql_ctx, &execution_plan)
                .await;

            insta::assert_snapshot!(name.clone(), executor);
        }
    }

    // let runtime = crate::cli::runtime::init(&Blueprint::default());
    // let req_ctx = RequestContext::new(runtime);
    // let graphql_ctx = EmptyResolverContext {};
    // let result = execution_plan.execute(&req_ctx, &graphql_ctx).await;

    // // TODO: remove error check
    // if let Ok(result) = result {
    //     insta::assert_json_snapshot!(name.clone(), result);
    // }
}
