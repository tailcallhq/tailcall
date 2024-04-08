use crate::query_plan::plan::{FieldTreeEntry, OperationPlan};

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

                match &tree.entry {
                    FieldTreeEntry::Compound(children) | FieldTreeEntry::CompoundList(children) => {
                        for tree in children.values() {
                            new_queue.push(tree);
                        }
                    }
                    _ => {}
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
