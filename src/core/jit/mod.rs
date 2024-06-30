mod builder;
mod exec;
mod model;
mod store;
mod synth;
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
use synth::{ConstValueSynth, Synthesizer};

use super::blueprint::Blueprint;
use super::ir::model::IR;
use super::json::JsonLike;

#[async_trait::async_trait]
trait Jit: Sized {
    type Input: Send + Sync;
    type Output: JsonLike<Output = Self::Output> + Send + Sync;
    type Error: Send + Sync;
    type Synth: Synthesizer<Value = std::result::Result<Self::Output, Self::Error>> + Send + Sync;
    type Exec: IRExecutor<Input = Self::Input, Output = Self::Output, Error = Self::Error>
        + Send
        + Sync;

    fn synth(&self) -> Self::Synth;
    fn exec(&self) -> Self::Exec;
    fn plan(self) -> ExecutionPlan;

    async fn execute(self, request: Request<Self::Input>) -> Response<Self::Output, Self::Error> {
        let synth = self.synth();
        let exec = self.exec();
        let plan = self.plan();
        let exe = Executor::new(plan, synth, exec);
        exe.execute(request).await
    }
}

pub struct ConstValueJit {
    plan: ExecutionPlan,
}

impl ConstValueJit {
    pub fn new(blueprint: Blueprint, request: Request<Value>) -> Result<Self> {
        Ok(Self { plan: request.try_plan_from(blueprint)? })
    }
}

#[async_trait::async_trait]
impl Jit for ConstValueJit {
    type Input = Value;
    type Output = Value;
    type Error = Error;
    type Synth = ConstValueSynth;
    type Exec = ConstValueExec;

    fn synth(&self) -> Self::Synth {
        ConstValueSynth::new()
    }

    fn exec(&self) -> Self::Exec {
        ConstValueExec::new()
    }

    fn plan(self) -> ExecutionPlan {
        self.plan
    }
}

struct ConstValueExec;
impl ConstValueExec {
    pub fn new() -> Self {
        Self
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
        unimplemented!()
    }
}
