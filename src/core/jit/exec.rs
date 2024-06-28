use crate::core::app_context::AppContext;
use super::ExecutionPlan;
use crate::core::runtime::TargetRuntime;

struct Exec {
    app_ctx: AppContext,
    plan: ExecutionPlan,
}

impl Exec {
    pub fn new(app_ctx: AppContext, plan: ExecutionPlan) -> Self {
        Self { app_ctx, plan }
    }
}
