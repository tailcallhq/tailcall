use std::borrow::Borrow;
use std::mem;
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;

use super::{Children, ExecutionPlan, Field, Request, Store};
use crate::core::ir::model::IR;
use crate::core::json::JsonLike;

pub struct Eval<Synth, Exec> {
    plan: ExecutionPlan,
    synth: Synth,
    exec: Exec,
}

impl<Input, Output, Synth, Exec> Eval<Synth, Exec>
where
    Output: Default + JsonLike<Output = Output>,
    Synth: Synthesizer<Value = Output>,
    Exec: Executor<Input = Input, Output = Output>,
{
    pub fn new(plan: ExecutionPlan, synth: Synth, exec: Exec) -> Self {
        Self { plan, synth, exec }
    }

    async fn execute_inner(&self, request: Request<Input>) -> Store<Output> {
        let store = Arc::new(Mutex::new(Store::new(self.plan.size())));
        let mut ctx = GraphQLContext::new(request, store.clone(), self.plan.to_owned(), &self.exec);
        ctx.execute().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new(0));
        store
    }

    pub async fn execute(&self, request: Request<Input>) -> Output {
        let store = self.execute_inner(request).await;
        self.synth.synthesize(&store)
    }
}

struct GraphQLContext<'a, Input, Output, Exec> {
    request: Request<Input>,
    store: Arc<Mutex<Store<Output>>>,
    plan: ExecutionPlan,
    exec: &'a Exec,
}

impl<'a, Input, Output, Exec> GraphQLContext<'a, Input, Output, Exec>
where
    Output: Default + JsonLike<Output = Output>,
    Exec: Executor<Input = Input, Output = Output>,
{
    fn new(
        request: Request<Input>,
        store: Arc<Mutex<Store<Output>>>,
        plan: ExecutionPlan,
        exec: &'a Exec,
    ) -> Self {
        Self { request, store, plan, exec }
    }

    async fn execute(&mut self) {
        join_all(self.plan.as_children().iter().map(|field| async {
            let ctx: ResolverContext<Input, Output> = ResolverContext::new(&self.request);
            self.execute_field(field, &ctx, false).await
        }))
        .await;
    }

    async fn execute_field<'b>(
        &'b self,
        field: &'b Field<Children>,
        ctx: &'b ResolverContext<'b, Input, Output>,
        is_multi: bool,
    ) {
        if let Some(ir) = &field.ir {
            let value = self.exec.execute(ir, ctx).await;
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

            if is_multi {
                self.store.lock().unwrap().set_multiple(&field.id, value)
            } else {
                self.store.lock().unwrap().set_single(&field.id, value)
            };
        }
    }
}

struct ResolverContext<'a, Input, Output> {
    request: &'a Request<Input>,
    parent: Option<&'a Output>,
    value: Option<&'a Output>,
}

impl<'a, Input, Output> Clone for ResolverContext<'a, Input, Output> {
    fn clone(&self) -> Self {
        Self {
            request: self.request,
            parent: self.parent,
            value: self.value,
        }
    }
}

impl<'a, Input, Output> ResolverContext<'a, Input, Output> {
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
    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a ResolverContext<'a, Self::Input, Self::Output>,
    ) -> Self::Output;
}
