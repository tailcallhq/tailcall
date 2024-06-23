use super::ExecutionPlan;
use crate::core::runtime::TargetRuntime;

struct Exec {
    runtime: TargetRuntime,
    plan: ExecutionPlan,
}

impl Exec {
    pub fn new(runtime: TargetRuntime, plan: ExecutionPlan) -> Self {
        Self { runtime, plan }
    }
}
