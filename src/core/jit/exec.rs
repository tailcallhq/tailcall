use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

use derive_getters::Getters;
use futures_util::future::join_all;

use super::context::Context;
use super::{DataPath, Field, LocationError, Nested, OperationPlan, Request, Response, Store};
use crate::core::ir::model::IR;
use crate::core::jit::synth::Synth;
use crate::core::jit::{self};
use crate::core::json::JsonLike;

type SharedStore<Output, Error> = Arc<Mutex<Store<Result<Output, Error>>>>;

#[derive(Debug, Clone)]
pub struct ExecutionEnv {
    err: Arc<Mutex<Vec<LocationError<jit::error::Error>>>>,
}

impl ExecutionEnv {
    pub fn new() -> Self {
        Self { err: Arc::new(Mutex::new(vec![])) }
    }
    pub fn add_error(&self, error: LocationError<jit::error::Error>) {
        self.err.lock().unwrap().push(error);
    }

    pub fn errors(&self) -> Vec<LocationError<jit::error::Error>> {
        self.err.lock().unwrap().clone()
    }
}

///
/// Default GraphQL executor that takes in a GraphQL Request and produces a
/// GraphQL Response
pub struct Executor<IRExec, Input> {
    plan: OperationPlan<Input>,
    exec: IRExec,
    env: ExecutionEnv,
}

impl<Input, Output, Exec> Executor<Exec, Input>
where
    Output: for<'a> JsonLike<'a> + Debug + Clone,
    Input: Clone + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = jit::Error>,
{
    pub fn new(plan: OperationPlan<Input>, exec: Exec) -> Self {
        Self { plan, exec, env: ExecutionEnv::new() }
    }

    pub async fn store(&self, request: Request<Input>) -> Store<Result<Output, jit::Error>> {
        let store = Arc::new(Mutex::new(Store::new()));
        let mut ctx = ExecutorInner::new(
            request,
            store.clone(),
            self.plan.to_owned(),
            &self.exec,
            &self.env,
        );
        ctx.init().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new());
        store
    }

    pub async fn execute(self, synth: Synth<Output>) -> Response<Output, jit::Error> {
        let mut response = Response::new(synth.synthesize());
        response.add_errors(self.env.errors());

        response
    }
}

#[derive(Getters)]
struct ExecutorInner<'a, Input, Output, Error, Exec> {
    request: Request<Input>,
    store: SharedStore<Output, Error>,
    plan: OperationPlan<Input>,
    ir_exec: &'a Exec,
    env: &'a ExecutionEnv,
}

impl<'a, Input, Output, Error, Exec> ExecutorInner<'a, Input, Output, Error, Exec>
where
    Output: for<'i> JsonLike<'i> + Debug,
    Input: Clone + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = Error>,
{
    fn new(
        request: Request<Input>,
        store: SharedStore<Output, Error>,
        plan: OperationPlan<Input>,
        ir_exec: &'a Exec,
        env: &'a ExecutionEnv,
    ) -> Self {
        Self { request, store, plan, ir_exec, env }
    }

    async fn init(&mut self) {
        join_all(self.plan.as_nested().iter().map(|field| async {
            let mut arg_map = indexmap::IndexMap::new();
            for arg in field.args.iter() {
                let name = arg.name.as_str();
                let value: Option<Input> = arg
                    .value
                    .clone()
                    // TODO: default value resolution should happen in the InputResolver
                    .or_else(|| arg.default_value.clone());

                if let Some(value) = value {
                    arg_map.insert(name, value);
                } else if !arg.type_of.is_nullable() {
                    // TODO: throw error here
                    todo!()
                }
            }
            let ctx = Context::new(&self.request, field, self.plan.is_query(), self.env)
                .with_args(arg_map);
            self.execute(field, &ctx, DataPath::new()).await
        }))
        .await;
    }

    async fn execute<'b>(
        &'b self,
        field: &'b Field<Nested<Input>, Input>,
        ctx: &'b Context<'b, Input, Output>,
        data_path: DataPath,
    ) -> Result<(), Error> {
        if let Some(ir) = &field.ir {
            let result = self.ir_exec.execute(ir, ctx).await;

            if let Ok(ref value) = result {
                // Array
                // Check if the field expects a list
                if field.type_of.is_list() {
                    // Check if the value is an array
                    if let Some(array) = value.as_array() {
                        join_all(field.nested_iter().map(|field| {
                            join_all(array.iter().enumerate().map(|(index, value)| {
                                let new_value = value.get_key(&field.name).unwrap_or(value);
                                let ctx = ctx.with_value_and_field(new_value, field);
                                let data_path = data_path.clone().with_index(index);
                                async move { self.execute(field, &ctx, data_path).await }
                            }))
                        }))
                        .await;
                    }
                    // TODO:  We should throw an error stating that we expected
                    // a list type here but because the `Error` is a
                    // type-parameter, its not possible
                }
                // TODO: Validate if the value is an Object
                // Has to be an Object, we don't do anything while executing if its a Scalar
                else {
                    join_all(field.nested_iter().map(|child| {
                        let new_value = value.get_key(&child.name).unwrap_or(value);
                        let ctx = ctx.with_value_and_field(new_value, child);
                        let data_path = data_path.clone();
                        async move { self.execute(child, &ctx, data_path).await }
                    }))
                    .await;
                }
            }

            let mut store = self.store.lock().unwrap();

            store.set(&field.id, &data_path, result);
        } else {
            // if the present field doesn't have IR, still go through it's extensions to see
            // if they've IR.
            join_all(field.nested_iter().map(|child| {
                let value = ctx.value().map(|v| v.get_key(&child.name).unwrap_or(v));
                let ctx = if let Some(v) = value {
                    ctx.with_value_and_field(v, child)
                } else {
                    ctx.with_field(child)
                };
                let data_path = data_path.clone();
                async move { self.execute(child, &ctx, data_path).await }
            }))
            .await;
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
