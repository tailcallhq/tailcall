use std::cell::{OnceCell, RefCell};
use std::fmt::{Debug, Formatter};
use std::thread;

use async_graphql_value::ConstValue;
use rquickjs::{Context, Ctx, FromJs, Function, IntoJs, Value};

use crate::cli::{Error, Result};
use crate::core::worker::{Command, Event, WorkerRequest};
use crate::core::{blueprint, WorkerIO};

struct LocalRuntime(Context);

thread_local! {
    // Practically only one JS runtime is created for every Runtime because tokio_runtime is single threaded.
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = const { RefCell::new(OnceCell::new()) };
}

#[rquickjs::function]
fn qjs_print(msg: String, is_err: bool) {
    if is_err {
        tracing::error!("{msg}");
    } else {
        tracing::info!("{msg}");
    }
}

fn setup_builtins(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    ctx.globals().set("__qjs_print", js_qjs_print)?;
    let _: Value = ctx.eval_file("src/cli/javascript/shim/console.js")?;

    Ok(())
}

impl LocalRuntime {
    fn try_new(script: blueprint::Script) -> Result<Self> {
        let source = script.source;
        let js_runtime = rquickjs::Runtime::new()?;
        let context = Context::full(&js_runtime)?;
        context.with(|ctx| {
            setup_builtins(&ctx)?;
            ctx.eval(source)
        })?;

        tracing::debug!("JS Runtime created: {:?}", thread::current().name());
        Ok(Self(context))
    }
}

pub struct Runtime {
    script: blueprint::Script,
    // Single threaded JS runtime, that's shared across all tokio workers.
    tokio_runtime: Option<tokio::runtime::Runtime>,
}

impl Debug for Runtime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Runtime {{ script: {:?} }}", self.script)
    }
}

impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .build()
            .expect("JS runtime not initialized");

        Self { script, tokio_runtime: Some(tokio_runtime) }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // implicit call implementation to shutdown the tokio runtime
        // without blocking. Otherwise it will panic on an attempt to
        // drop AppContext in async runtime (e.g. in tests at least)
        if let Some(runtime) = self.tokio_runtime.take() {
            runtime.shutdown_background();
        }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Event, Command> for Runtime {
    async fn call(&self, name: &str, event: Event) -> Result<Option<Command>> {
        let script = self.script.clone();
        let name = name.to_string(); // TODO
        if let Some(runtime) = &self.tokio_runtime {
            runtime
                .spawn(async move {
                    init_rt(script)?;
                    call(name, event)
                })
                .await?
        } else {
            Err(Error::JsRuntimeStoppedError)
        }
    }
}

#[async_trait::async_trait]
impl WorkerIO<ConstValue, ConstValue> for Runtime {
    async fn call(&self, name: &str, input: ConstValue) -> Result<Option<ConstValue>> {
        let script = self.script.clone();
        let name = name.to_string();
        let value = serde_json::to_string(&input)?;
        if let Some(runtime) = &self.tokio_runtime {
            runtime
                .spawn(async move {
                    init_rt(script)?;
                    execute_inner(name, value).map(Some)
                })
                .await?
        } else {
            Err(Error::JsRuntimeStoppedError)
        }
    }
}

fn init_rt(script: blueprint::Script) -> Result<()> {
    // initialize runtime if this is the first call
    // exit if failed to initialize
    LOCAL_RUNTIME.with(move |cell| {
        if cell.borrow().get().is_none() {
            LocalRuntime::try_new(script).and_then(|runtime| {
                cell.borrow()
                    .set(runtime)
                    .map_err(|_| Error::ReinitializeQuickjsRuntimeError)
            })
        } else {
            Ok(())
        }
    })
}

fn prepare_args<'js>(ctx: &Ctx<'js>, req: WorkerRequest) -> rquickjs::Result<(Value<'js>,)> {
    let object = rquickjs::Object::new(ctx.clone())?;
    object.set("request", req.into_js(ctx)?)?;
    Ok((object.into_value(),))
}

fn call(name: String, event: Event) -> Result<Option<Command>> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell.get_mut().ok_or(Error::RuntimeNotInitializedError)?;
        runtime.0.with(|ctx| match event {
            Event::Request(req) => {
                let fn_as_value = ctx.globals().get::<&str, Function>(name.as_str())?;

                let function = fn_as_value.as_function().ok_or(Error::NotaFunctionError)?;

                let args = prepare_args(&ctx, req)?;
                let command: Option<Value> = function.call(args).ok();
                command
                    .map(|output| Command::from_js(&ctx, output))
                    .transpose()
                    .map_err(|_| Error::DeserializeFailed)
            }
        })
    })
}

fn execute_inner(name: String, value: String) -> Result<ConstValue> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell.get_mut().ok_or(Error::RuntimeNotInitializedError)?;
        runtime.0.with(|ctx| {
            let fn_as_value = ctx.globals().get::<_, rquickjs::Function>(&name)?;

            let function = fn_as_value.as_function().ok_or(Error::NotaFunctionError)?;
            let val: String = function.call((value,))?;
            Ok::<_, Error>(serde_json::from_str(&val)?)
        })
    })
}
