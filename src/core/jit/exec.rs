use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

use derive_getters::Getters;
use futures_util::future::join_all;

use super::context::{Context, RequestContext};
use super::{DataPath, OperationPlan, Positioned, Response, Store};
use crate::core::ir::model::IR;
use crate::core::ir::TypeName;
use crate::core::jit;
use crate::core::jit::synth::Synth;
use crate::core::json::{JsonLike, JsonObjectLike};

type SharedStore<Output, Error> = Arc<Mutex<Store<Result<TypedValue<Output>, Positioned<Error>>>>>;

///
/// Default GraphQL executor that takes in a GraphQL Request and produces a
/// GraphQL Response
pub struct Executor<IRExec, Input> {
    ctx: RequestContext<Input>,
    exec: IRExec,
}

impl<Input, Output, Exec> Executor<Exec, Input>
where
    Output:
        for<'b> JsonLike<'b, JsonObject<'b>: JsonObjectLike<'b, Value = Output>> + Debug + Clone,
    Input: Clone + Debug,
    Exec: IRExecutor<Input = Input, Output = Output, Error = jit::Error>,
{
    pub fn new(plan: OperationPlan<Input>, exec: Exec) -> Self {
        Self { exec, ctx: RequestContext::new(plan) }
    }

    pub async fn store(&self) -> Store<Result<TypedValue<Output>, Positioned<jit::Error>>> {
        let store = Arc::new(Mutex::new(Store::new()));
        let mut ctx = ExecutorInner::new(store.clone(), &self.exec, &self.ctx);
        ctx.init().await;

        let store = mem::replace(&mut *store.lock().unwrap(), Store::new());
        store
    }

    pub async fn execute(self, synth: Synth<Output>) -> Response<Output, jit::Error> {
        let mut response = Response::new(synth.synthesize());
        response.add_errors(self.ctx.errors().clone());
        response
    }
}

#[derive(Getters)]
struct ExecutorInner<'a, Input, Output, Error, Exec> {
    store: SharedStore<Output, Error>,
    ir_exec: &'a Exec,
    request: &'a RequestContext<Input>,
}

impl<'a, Input, Output, Error, Exec> ExecutorInner<'a, Input, Output, Error, Exec>
where
    Output: for<'i> JsonLike<'i> + Debug,
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
            // TODO: with_args should be called on inside iter_field on any level, not only
            // for root fields
            let ctx = Context::new(field, self.request).with_args(arg_map);
            self.execute(&ctx, DataPath::new()).await
        }))
        .await;
    }

    async fn iter_field<'b>(
        &'b self,
        ctx: &'b Context<'b, Input, Output>,
        data_path: &DataPath,
        result: TypedValueRef<'b, Output>,
    ) -> Result<(), Error> {
        let field = ctx.field();
        let TypedValueRef { value, type_name } = result;
        // Array
        // Check if the field expects a list
        if field.type_of.is_list() {
            // Check if the value is an array
            if let Some(array) = value.as_array() {
                join_all(array.iter().enumerate().map(|(index, value)| {
                    let type_name = match &type_name {
                        Some(TypeName::Single(type_name)) => type_name, /* TODO: should throw */
                        // ValidationError
                        Some(TypeName::Vec(v)) => &v[index],
                        None => field.type_of.name(),
                    };
                    join_all(field.nested_iter(type_name).map(|field| {
                        let ctx = ctx.with_value_and_field(value, field);
                        let data_path = data_path.clone().with_index(index);
                        async move { self.execute(&ctx, data_path).await }
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
            let type_name = match &type_name {
                Some(TypeName::Single(type_name)) => type_name,
                Some(TypeName::Vec(_)) => panic!("TypeName type mismatch"), /* TODO: should throw ValidationError */
                None => field.type_of.name(),
            };

            join_all(field.nested_iter(type_name).map(|child| {
                let ctx = ctx.with_value_and_field(value, child);
                let data_path = data_path.clone();
                async move { self.execute(&ctx, data_path).await }
            }))
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

            if let Ok(ref result) = result {
                self.iter_field(ctx, &data_path, result.as_ref()).await?;
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
                .and_then(|v| v.get_key(&field.name))
                // in case there is no value we still put some dumb empty value anyway
                // to force execution of the nested fields even when parent object is not present.
                // For async_graphql it's done by `fix_dangling_resolvers` fn that basically creates
                // fake IR that resolves to empty object. The `fix_dangling_resolvers` is also
                // working here, but eventually it can be replaced by this logic
                // here without doing the "fix"
                .unwrap_or(&default_obj);

            let result = TypedValueRef { value, type_name: None };

            self.iter_field(ctx, &data_path, result).await?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct TypedValue<V> {
    pub value: V,
    pub type_name: Option<TypeName>,
}

pub struct TypedValueRef<'a, V> {
    pub value: &'a V,
    pub type_name: Option<&'a TypeName>,
}

impl<V> TypedValue<V> {
    pub fn new(value: V) -> Self {
        Self { value, type_name: None }
    }

    pub fn as_ref(&self) -> TypedValueRef<'_, V> {
        TypedValueRef { value: &self.value, type_name: self.type_name.as_ref() }
    }
}

impl<'a, V> TypedValueRef<'a, V> {
    pub fn new(value: &'a V) -> Self {
        Self { value, type_name: None }
    }

    pub fn map<'out, U>(&self, map: impl FnOnce(&V) -> &'out U) -> TypedValueRef<'out, U>
    where
        'a: 'out,
    {
        TypedValueRef { value: map(self.value), type_name: self.type_name }
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
    ) -> Result<TypedValue<Self::Output>, Self::Error>;
}
