use crate::query_plan::plan::{FieldTree, FieldTreeEntry, OperationPlan};

use super::execution::ExecutionStep;

pub struct SimpleExecutionBuilder {}

impl SimpleExecutionBuilder {
    fn inner_build(&self, tree: &FieldTree) -> ExecutionStep {
        let mut steps = Vec::new();

        match &tree.entry {
            FieldTreeEntry::Compound(children) | FieldTreeEntry::CompoundList(children) => {
                for tree in children.values() {
                    steps.push(self.inner_build(tree));
                }
            }
            _ => {}
        }

        let steps = ExecutionStep::Parallel(steps);

        if let Some(field_plan_id) = &tree.field_plan_id {
            ExecutionStep::Sequential(vec![ExecutionStep::Resolve(*field_plan_id), steps])
        } else {
            steps
        }
    }

    pub fn build(&self, operation_plan: &OperationPlan) -> ExecutionStep {
        self.inner_build(&operation_plan.field_tree).flatten()
    }
}
