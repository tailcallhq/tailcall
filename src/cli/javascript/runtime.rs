use std::cell::{OnceCell, RefCell};
use std::thread;

use rquickjs::{Context, Ctx, FromJs, Function, IntoJs, Value};

use super::request_filter::{Command, Event};
use super::JsRequest;
use crate::core::{blueprint, WorkerIO};

struct LocalRuntime(Context);

thread_local! {
    // Practically only one JS runtime is created because CHANNEL_RUNTIME is single threaded.
    // TODO: that is causing issues in `execution_spec` tests because the runtime
    // is initialized only once and that implementation will be reused by all the tests
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = const { RefCell::new(OnceCell::new()) };
}

// Single threaded JS runtime, that's shared across all tokio workers.
lazy_static::lazy_static! {
    static ref CHANNEL_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .build()
        .expect("JS runtime not initialized");
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
    fn try_new(script: blueprint::Script) -> anyhow::Result<Self> {
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
}

impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        Self { script }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Event, Command> for Runtime {
    async fn call(&self, name: String, event: Event) -> anyhow::Result<Option<Command>> {
        let script = self.script.clone();
        CHANNEL_RUNTIME
            .spawn(async move {
                // initialize runtime if this is the first call
                // exit if failed to initialize
                LOCAL_RUNTIME.with(move |cell| {
                    if cell.borrow().get().is_none() {
                        LocalRuntime::try_new(script).and_then(|runtime| {
                            cell.borrow().set(runtime).map_err(|_| {
                                anyhow::anyhow!(
                                    "trying to reinitialize an already initialized QuickJS runtime"
                                )
                            })
                        })
                    } else {
                        Ok(())
                    }
                })?;

                call(name, event)
            })
            .await?
    }
}

fn prepare_args<'js>(ctx: &Ctx<'js>, req: JsRequest) -> rquickjs::Result<(Value<'js>,)> {
    let object = rquickjs::Object::new(ctx.clone())?;
    object.set("request", req.into_js(ctx)?)?;
    Ok((object.into_value(),))
}

fn call(name: String, event: Event) -> anyhow::Result<Option<Command>> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))?;
        runtime.0.with(|ctx| match event {
            Event::Request(req) => {
                let fn_as_value = ctx
                    .globals()
                    .get::<&str, Function>(name.as_str())
                    .map_err(|_| anyhow::anyhow!("globalThis not initialized"))?;

                let function = fn_as_value
                    .as_function()
                    .ok_or(anyhow::anyhow!("`{name}` is not a function"))?;

                let args = prepare_args(&ctx, req)?;
                let command: Option<Value> = function.call(args).ok();
                command
                    .map(|output| Command::from_js(&ctx, output))
                    .transpose()
                    .map_err(|e| anyhow::anyhow!("deserialize failed: {e}"))
            }
        })
    })
}
