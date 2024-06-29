use std::borrow::Borrow;
use std::mem;
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;

use super::{Children, ExecutionPlan, Field, Request, Store, Synthesizer};
use crate::core::ir::model::IR;
use crate::core::json::JsonLike;
use crate::core::runtime::TargetRuntime;

pub struct Eval<Synth> {
    runtime: TargetRuntime,
    plan: ExecutionPlan,
    synth: Synth,
}

impl<Output, Synth> Eval<Synth>
where
    Output: Default + JsonLike<Output = Output>,
    Synth: Synthesizer<Value = Output>,
{
    pub fn new(runtime: TargetRuntime, plan: ExecutionPlan, synth: Synth) -> Self {
        Self { runtime, plan, synth }
    }

    async fn execute_inner<Input>(&self, request: Request<Input>) -> Store<Output> {
        let store = Arc::new(Mutex::new(Store::new(self.plan.size())));
        let mut ctx = GraphQLContext::new(request, store.clone(), self.plan.to_owned());
        ctx.execute().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new(0));
        store
    }

    pub async fn execute<Input>(&self, request: Request<Input>) -> Output {
        let store = self.execute_inner(request).await;
        self.synth.synthesize(&store)
    }
}

struct GraphQLContext<Input, Output> {
    request: Request<Input>,
    store: Arc<Mutex<Store<Output>>>,
    plan: ExecutionPlan,
}

impl<Input, Output: Default + JsonLike<Output = Output>> GraphQLContext<Input, Output> {
    fn new(request: Request<Input>, store: Arc<Mutex<Store<Output>>>, plan: ExecutionPlan) -> Self {
        Self { request, store, plan }
    }

    async fn execute(&mut self) {
        join_all(self.plan.as_children().iter().map(|field| async {
            let ctx: ResolverContext<Input, Output> = ResolverContext::new(&self.request);
            self.execute_field(field, &ctx, false).await
        }))
        .await;
    }

    async fn execute_field<'a>(
        &'a self,
        field: &'a Field<Children>,
        ctx: &'a ResolverContext<'a, Input, Output>,
        is_multi: bool,
    ) {
        if let Some(ir) = &field.ir {
            let value = self.execute_ir(ir, ctx).await;
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

    async fn execute_ir<'a>(
        &'a self,
        _ir: &'a IR,
        _ctx: &'a ResolverContext<'a, Input, Output>,
    ) -> Output {
        todo!();
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
