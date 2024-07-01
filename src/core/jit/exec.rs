use std::borrow::Borrow;
use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

use derive_getters::Getters;
use futures_util::future::join_all;

use super::context::Context;
use super::synth::Synthesizer;
use super::{Children, ExecutionPlan, Field, Request, Response, Store};
use crate::core::ir::model::IR;
use crate::core::json::JsonLike;

///
/// Default GraphQL executor that takes in a GraphQL Request and produces a
/// GraphQL Response
pub struct Executor<Synth, IRExec> {
    plan: ExecutionPlan,
    synth: Synth,
    exec: IRExec,
}

impl<Input, Output, Error, Synth, Exec> Executor<Synth, Exec>
where
    Output: JsonLike<Output = Output> + Default + Debug,
    Synth: Synthesizer<Value = Result<Output, Error>>,
    Exec: IRExecutor<Input = Input, Output = Output, Error = Error>,
    Error: Debug,
{
    pub fn new(plan: ExecutionPlan, synth: Synth, exec: Exec) -> Self {
        Self { plan, synth, exec }
    }

    async fn execute_inner(&self, request: Request<Input>) -> Store<Result<Output, Error>> {
        let store: Arc<Mutex<Store<Result<Output, Error>>>> =
            Arc::new(Mutex::new(Store::new(self.plan.size())));
        let mut ctx = ExecutorInner::new(request, store.clone(), self.plan.to_owned(), &self.exec);
        ctx.execute().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new(0));
        store
    }

    pub async fn execute(self, request: Request<Input>) -> Response<Output, Error> {
        let store = self.execute_inner(request).await;
        Response::new(self.synth.synthesize(store))
    }
}

#[derive(Getters)]
struct ExecutorInner<'a, Input, Output, Error, Exec> {
    request: Request<Input>,
    store: Arc<Mutex<Store<Result<Output, Error>>>>,
    plan: ExecutionPlan,
    exec: &'a Exec,
}

impl<'a, Input, Output, Error, Exec> ExecutorInner<'a, Input, Output, Error, Exec>
where
    Output: JsonLike<Output = Output> + Default + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = Error>,
    Error: Debug,
{
    fn new(
        request: Request<Input>,
        store: Arc<Mutex<Store<Result<Output, Error>>>>,
        plan: ExecutionPlan,
        exec: &'a Exec,
    ) -> Self {
        Self { request, store, plan, exec }
    }

    async fn execute(&mut self) {
        join_all(self.plan.as_children().iter().map(|field| async {
            let ctx = Context::new(&self.request);
            self.execute_field(field, &ctx, false).await
        }))
        .await;
    }

    async fn execute_field<'b>(
        &'b self,
        field: &'b Field<Children>,
        ctx: &'b Context<'b, Input, Output>,
        is_multi: bool,
    ) -> Result<(), Error> {
        if let Some(ir) = &field.ir {
            let result = self.exec.execute(ir, ctx).await;
            if let Ok(ref value) = result {
                // Array
                if let Ok(array) = value.as_array_ok() {
                    let values = join_all(field.children().iter().map(|child| {
                        join_all(array.iter().map(|value| {
                            let ctx = ctx.with_value(value);

                            // TODO: doesn't handle nested values
                            async move {
                                if let Some(ir) = &child.ir {
                                    self.exec.execute(ir, &ctx).await
                                } else {
                                    Ok(Output::default())
                                }
                            }
                        }))
                    }))
                    .await;

                    let mut store = self.store.lock().unwrap();
                    for (field, values) in field.children().iter().zip(values) {
                        store.set_multiple(&field.id, values)
                    }

                // Object
                } else {
                    join_all(field.children().iter().map(|child| {
                        let ctx = ctx.with_value(value);
                        async move { self.execute_field(child, &ctx, false).await }
                    }))
                    .await;
                }
            }

            self.store.lock().unwrap().set_single(&field.id, result)
        }
        Ok(())
    }
}

/// Executor for IR
#[async_trait::async_trait]
pub trait IRExecutor {
    type Input;
    type Output;
    type Error;
    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<Self::Output, Self::Error>;
}
