use std::borrow::Borrow;
use std::mem;
use std::sync::{Arc, Mutex};

use derive_getters::Getters;
use futures_util::future::join_all;

use super::{Children, ExecutionPlan, Field, Request, Store};
use crate::core::ir::model::IR;
use crate::core::json::JsonLike;

pub struct QueryExecutor<Synth, Exec> {
    plan: ExecutionPlan,
    synth: Synth,
    exec: Exec,
}

impl<Input, Output, Error, Synth, Exec> QueryExecutor<Synth, Exec>
where
    Output: JsonLike<Output = Output>,
    Synth: Synthesizer<Value = Result<Output, Error>>,
    Exec: Executor<Input = Input, Output = Output, Error = Error>,
{
    pub fn new(plan: ExecutionPlan, synth: Synth, exec: Exec) -> Self {
        Self { plan, synth, exec }
    }

    async fn execute_inner(&self, request: Request<Input>) -> Store<Result<Output, Error>> {
        let store: Arc<Mutex<Store<Result<Output, Error>>>> =
            Arc::new(Mutex::new(Store::new(self.plan.size())));
        let mut ctx =
            QueryExecutorInner::new(request, store.clone(), self.plan.to_owned(), &self.exec);
        ctx.execute().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new(0));
        store
    }

    pub async fn execute(&self, request: Request<Input>) -> Result<Output, Error> {
        let store = self.execute_inner(request).await;
        self.synth.synthesize(&store)
    }
}

#[derive(Getters)]
pub struct QueryExecutorInner<'a, Input, Output, Error, Exec> {
    request: Request<Input>,
    store: Arc<Mutex<Store<Result<Output, Error>>>>,
    plan: ExecutionPlan,
    exec: &'a Exec,
}

impl<'a, Input, Output, Error, Exec> QueryExecutorInner<'a, Input, Output, Error, Exec>
where
    Output: JsonLike<Output = Output>,
    Exec: Executor<Input = Input, Output = Output, Error = Error>,
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
            let ctx = EvaluationContext::new(&self.request);
            self.execute_field(field, &ctx, false).await
        }))
        .await;
    }

    async fn execute_field<'b>(
        &'b self,
        field: &'b Field<Children>,
        ctx: &'b EvaluationContext<'b, Input, Output>,
        is_multi: bool,
    ) -> Result<(), Error> {
        if let Some(ir) = &field.ir {
            let result = self.exec.execute(ir, ctx).await;
            if let Ok(ref value) = result {
                // Array
                if let Ok(array) = value.as_array_ok() {
                    let ctx = ctx.with_parent_value(&value);
                    join_all(array.iter().map(|value| {
                        let ctx = ctx.with_value(value);

                        join_all(field.children().iter().map(|child| {
                            let ctx = ctx.clone();
                            async move {
                                let ctx = ctx.clone();
                                self.execute_field(child, ctx.clone().borrow(), true).await
                            }
                        }))
                    }))
                    .await;

                // Object
                } else {
                    join_all(field.children().iter().map(|child| {
                        let ctx = ctx.clone();
                        let value = &value;
                        async move {
                            let ctx = ctx.with_parent_value(value);
                            self.execute_field(child, &ctx, false).await
                        }
                    }))
                    .await;
                }
            }

            if is_multi {
                self.store.lock().unwrap().set_multiple(&field.id, result)
            } else {
                self.store.lock().unwrap().set_single(&field.id, result)
            };
        }
        Ok(())
    }
}

#[derive(Getters)]
pub struct EvaluationContext<'a, Input, Output> {
    request: &'a Request<Input>,
    parent: Option<&'a Output>,
    value: Option<&'a Output>,
}

impl<'a, Input, Output> Clone for EvaluationContext<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: self.value,
        }
    }
}

impl<'a, Input, Output> EvaluationContext<'a, Input, Output> {
    fn new(request: &'a Request<Input>) -> Self {
        Self { request, parent: None, value: None }
    }

    fn with_parent_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: Some(value),
        }
    }

    fn with_value(&self, value: &'a Output) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: Some(value),
        }
    }
}

pub trait Synthesizer {
    type Value;
    fn synthesize(&self, store: &Store<Self::Value>) -> Self::Value;
}

#[async_trait::async_trait]
pub trait Executor {
    type Input;
    type Output;
    type Error;
    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a EvaluationContext<'a, Self::Input, Self::Output>,
    ) -> Result<Self::Output, Self::Error>;
}
