mod builder;
mod exec;
mod model;
mod store;
mod synth;
use std::sync::Arc;

use async_graphql::Value;
use builder::*;
use context::Context;
use exec::{Executor, IRExecutor};
use model::*;
use store::*;
mod context;
mod error;
mod request;
mod response;

// NOTE: Only used in tests and benchmarks
pub mod common;
pub use error::*;
pub use request::*;
pub use response::*;
use synth::{SynthConst, Synthesizer};

use super::app_context::AppContext;
use super::blueprint::Blueprint;
use super::http::RequestContext;
use super::ir::model::IR;
use super::ir::EvalContext;
use super::json::JsonLike;
use super::runtime::TargetRuntime;

pub struct ConstValueExecutor {
    plan: ExecutionPlan,
    app_ctx: Arc<AppContext>,
}

impl ConstValueExecutor {
    pub fn new(request: Request<Value>, app_ctx: Arc<AppContext>) -> Result<Self> {
        Ok(Self { plan: request.try_plan_from(&app_ctx.blueprint)?, app_ctx })
    }

    async fn execute(self, request: Request<Value>) -> Response<Value, Error> {
        let ctx = RequestContext::from(self.app_ctx.as_ref());
        let exec = ConstValueExec::new(ctx);
        let plan = self.plan;
        // TODO: drop the clones in plan
        let synth = SynthConst::new(plan.clone());
        let exe = Executor::new(plan, synth, exec);
        exe.execute(request).await
    }
}

struct ConstValueExec {
    req_context: RequestContext,
}

impl ConstValueExec {
    pub fn new(ctx: RequestContext) -> Self {
        Self { req_context: ctx }
    }
}

#[async_trait::async_trait]
impl IRExecutor for ConstValueExec {
    type Input = Value;
    type Output = Value;
    type Error = Error;

    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<Value> {
        let req_context = &self.req_context;
        let mut ctx = EvalContext::new(req_context, ctx);
        Ok(ir.eval(&mut ctx).await?)
    }
}
