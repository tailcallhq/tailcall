use super::{ExecutionPlan, Request, Response, Store, Synthesizer};
use crate::core::runtime::TargetRuntime;

struct Eval<Synth> {
    runtime: TargetRuntime,
    plan: ExecutionPlan,
    synth: Synth,
}

impl<'a, Synth> Eval<Synth> {
    pub fn new(runtime: TargetRuntime, plan: ExecutionPlan, synth: Synth) -> Self {
        Self { runtime, plan, synth }
    }

    async fn execute_inner<Input, Output, Error>(
        &self,
        _request: Request<Input>,
    ) -> Store<Result<Output, Error>> {
        todo!()
    }

    pub async fn execute<Input, Output, Error>(
        &'a self,
        request: Request<Input>,
    ) -> Response<Output, Error>
    where
        Synth: Synthesizer<Value = Result<Output, Error>>,
    {
        let store = self.execute_inner(request).await;
        let output = self.synth.synthesize(&store);
        Response::from_result(output)
    }
}
