use std::marker::PhantomData;

use super::{ExecutionPlan, Request, Response};
use crate::core::runtime::TargetRuntime;

struct Eval<Input, Output, Error> {
    runtime: TargetRuntime,
    plan: ExecutionPlan,
    _output: PhantomData<Output>,
    _input: PhantomData<Input>,
    _error: PhantomData<Error>,
}

impl<Input, Output, Error> Eval<Input, Output, Error> {
    pub fn new(runtime: TargetRuntime, plan: ExecutionPlan) -> Self {
        Self {
            runtime,
            plan,
            _output: PhantomData::default(),
            _input: PhantomData::default(),
            _error: PhantomData::default(),
        }
    }

    pub async fn execute(&self, request: Request<Input>) -> Response<Output, Error> {
        todo!()
    }
}
