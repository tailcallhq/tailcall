use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

use derive_getters::Getters;
use futures_util::future::join_all;

use super::context::{Context, RequestContext};
use super::{DataPath, OperationPlan, Positioned, Response, Store};
use crate::core::ir::model::IR;
use crate::core::ir::TypedValue;
use crate::core::jit;
use crate::core::jit::synth::Synth;
use crate::core::json::{JsonLike, JsonObjectLike};

type SharedStore<Output, Error> = Arc<Mutex<Store<Result<Output, Positioned<Error>>>>>;

///
/// Default GraphQL executor that takes in a GraphQL Request and produces a
/// GraphQL Response
pub struct Executor<'a, IRExec, Input> {
    ctx: RequestContext<'a, Input>,
    exec: IRExec,
}

impl<'a, Input, Output, Exec> Executor<'a, Exec, Input>
where
    Output: for<'b> JsonLike<'b> + Debug + Clone,
    Input: Clone + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = jit::Error>,
{
    pub fn new(plan: &'a OperationPlan<Input>, exec: Exec) -> Self {
        Self { exec, ctx: RequestContext::new(plan) }
    }

    pub async fn store(&self) -> Store<Result<Output, Positioned<jit::Error>>> {
        let store = Arc::new(Mutex::new(Store::new()));
        let mut ctx = ExecutorInner::new(store.clone(), &self.exec, &self.ctx);
        ctx.init().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new());
        store
    }

    pub async fn execute(self, synth: Synth<'a, Output>) -> Response<Output, jit::Error> {
        let mut response = Response::new(synth.synthesize());
        response.add_errors(self.ctx.errors().clone());
        response
    }
}

#[derive(Getters)]
struct ExecutorInner<'a, Input, Output, Error, Exec> {
    store: SharedStore<Output, Error>,
    ir_exec: &'a Exec,
    request: &'a RequestContext<'a, Input>,
}

impl<'a, Input, Output, Error, Exec> ExecutorInner<'a, Input, Output, Error, Exec>
where
    for<'i> Output: JsonLike<'i> + TypedValue<'i> + Debug,
    Input: Clone + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = Error>,
{
    fn new(
        store: SharedStore<Output, Error>,
        ir_exec: &'a Exec,
        env: &'a RequestContext<Input>,
    ) -> Self {
        Self { store, ir_exec, request: env }
    }

    async fn init(&mut self) {
        join_all(self.request.plan().as_nested().iter().map(|field| async {
            let ctx = Context::new(field, self.request);
            // TODO: with_args should be called on inside iter_field on any level, not only
            // for root fields
            self.execute(&ctx, DataPath::new()).await
        }))
        .await;
    }

    async fn iter_field<'b>(
        &'b self,
        ctx: &'b Context<'b, Input, Output>,
        data_path: &DataPath,
        value: &'b Output,
    ) -> Result<(), Error> {
        let field = ctx.field();
        // Array
        // Check if the field expects a list
        if field.type_of.is_list() {
            // Check if the value is an array
            if let Some(array) = value.as_array() {
                join_all(array.iter().enumerate().map(|(index, value)| {
                    join_all(
                        self.request
                            .plan()
                            .field_iter_only(field, value)
                            .map(|field| {
                                let ctx = ctx.with_value_and_field(value, field);
                                let data_path = data_path.clone().with_index(index);
                                async move { self.execute(&ctx, data_path).await }
                            }),
                    )
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
            join_all(
                self.request
                    .plan()
                    .field_iter_only(field, value)
                    .map(|child| {
                        let ctx = ctx.with_value_and_field(value, child);
                        let data_path = data_path.clone();
                        async move { self.execute(&ctx, data_path).await }
                    }),
            )
            .await;
        }

        Ok(())
    }

    async fn execute<'b>(
        &'b self,
        ctx: &'b Context<'b, Input, Output>,
        data_path: DataPath,
    ) -> Result<(), Error> {
        let field = ctx.field();

        if let Some(ir) = &field.ir {
            let result = self.ir_exec.execute(ir, ctx).await;

            if let Ok(value) = &result {
                self.iter_field(ctx, &data_path, value).await?;
            }

            let mut store = self.store.lock().unwrap();

            store.set(
                &field.id,
                &data_path,
                result.map_err(|e| Positioned::new(e, field.pos)),
            );
        } else {
            // if the present field doesn't have IR, still go through it's extensions to see
            // if they've IR.
            let default_obj = Output::object(Output::JsonObject::new());
            let value = ctx
                .value()
                .and_then(|v| v.get_key(&field.output_name))
                // in case there is no value we still put some dumb empty value anyway
                // to force execution of the nested fields even when parent object is not present.
                // For async_graphql it's done by `fix_dangling_resolvers` fn that basically creates
                // fake IR that resolves to empty object. The `fix_dangling_resolvers` is also
                // working here, but eventually it can be replaced by this logic
                // here without doing the "fix"
                .unwrap_or(&default_obj);

            self.iter_field(ctx, &data_path, value).await?;
        }

        Ok(())
    }
}

/// Executor for IR
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
