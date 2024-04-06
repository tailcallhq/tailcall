use crate::query_plan::plan::OperationPlan;

use super::execution::ExecutionStep;

pub struct SimpleExecutionBuilder {}

impl SimpleExecutionBuilder {
    pub fn build(&self, operation_plan: &OperationPlan) -> ExecutionStep {
        let mut steps = Vec::new();
        let mut queue = vec![&operation_plan.field_tree];

        while !queue.is_empty() {
            let mut new_queue = Vec::new();
            let mut parallel_steps = Vec::new();

            for tree in queue {
                if let Some(field_plan_id) = &tree.field_plan_id {
                    parallel_steps.push(ExecutionStep::Resolve(*field_plan_id));
                }

                if let Some(children) = &tree.children {
                    for tree in children.values() {
                        new_queue.push(tree);
                    }
                }
            }

            if !parallel_steps.is_empty() {
                steps.push(ExecutionStep::parallel(parallel_steps));
            }
            queue = new_queue;
        }

        ExecutionStep::sequential(steps)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use async_graphql::parser::parse_query;

    use crate::{
        blueprint::Blueprint,
        config::{Config, ConfigModule},
        query_plan::{
            execution::simple::SimpleExecutionBuilder,
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

            insta::assert_snapshot!(name.clone(), execution_plan);
        }
    }
}
