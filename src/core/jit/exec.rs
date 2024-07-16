use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

use async_graphql::Positioned;
use derive_getters::Getters;
use futures_util::future::join_all;

use super::context::Context;
use super::synth::Synthesizer;
use super::{DataPath, Field, Nested, OperationPlan, Request, Response, Store};
use crate::core::ir::model::IR;
use crate::core::json::JsonLike;

type SharedStore<Output, Error> = Arc<Mutex<Store<Result<Output, Positioned<Error>>>>>;

///
/// Default GraphQL executor that takes in a GraphQL Request and produces a
/// GraphQL Response
pub struct Executor<Synth, IRExec, Input> {
    plan: OperationPlan<Input>,
    synth: Synth,
    exec: IRExec,
}

impl<Input, Output, Error, Synth, Exec> Executor<Synth, Exec, Input>
where
    Output: for<'a> JsonLike<'a> + Debug,
    Input: Clone + Debug,
    Synth: Synthesizer<Value = Result<Output, Positioned<Error>>, Variable = Input>,
    Exec: IRExecutor<Input = Input, Output = Output, Error = Error>,
{
    pub fn new(plan: OperationPlan<Input>, synth: Synth, exec: Exec) -> Self {
        Self { plan, synth, exec }
    }

    async fn execute_inner(
        &self,
        request: Request<Input>,
    ) -> Store<Result<Output, Positioned<Error>>> {
        let store = Arc::new(Mutex::new(Store::new()));
        let mut ctx = ExecutorInner::new(request, store.clone(), self.plan.to_owned(), &self.exec);
        ctx.init().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new());
        store
    }

    pub async fn execute(self, request: Request<Input>) -> Response<Output, Error> {
        let vars = request.variables.clone();
        let store = self.execute_inner(request).await;
        Response::new(self.synth.synthesize(store, vars))
    }
}

#[derive(Getters)]
struct ExecutorInner<'a, Input, Output, Error, Exec> {
    request: Request<Input>,
    store: SharedStore<Output, Error>,
    plan: OperationPlan<Input>,
    ir_exec: &'a Exec,
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
    ) -> Self {
        Self { request, store, plan, ir_exec }
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
            let ctx = Context::new(&self.request, self.plan.is_query(), field).with_args(arg_map);
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
                                let ctx = ctx.with_value(value).with_field(field); // Output::JsonArray::Value
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
                        let ctx = ctx.with_value(value).with_field(child);
                        let data_path = data_path.clone();
                        async move { self.execute(child, &ctx, data_path).await }
                    }))
                    .await;
                }
            }

            let mut store = self.store.lock().unwrap();

            store.set(
                &field.id,
                &data_path,
                result.map_err(|e| Positioned::new(e, field.pos)),
            );
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
