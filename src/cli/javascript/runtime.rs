use std::cell::{OnceCell, RefCell};
use std::thread;

use super::request_filter::{Command, Event};
use super::JsRequest;
use rquickjs::{Context, Ctx, FromJs, IntoJs, Value};
use crate::{blueprint, WorkerIO};

struct LocalRuntime {
    context: Context,

    // NOTE: This doesn't need to be accessed directly right now but context holds a
    // reference to it, so make sure that this is not dropped
    _js_runtime: rquickjs::Runtime,
}

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
        eprintln!("{msg}")
    } else {
        println!("{msg}")
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
        let js_runtime = rquickjs::Runtime::new().unwrap();
        let context = Context::full(&js_runtime).unwrap();
        context.with(|ctx| {
            setup_builtins(&ctx)?;
            ctx.eval(source)
        })?;

        tracing::debug!("JS Runtime created: {:?}", thread::current().name());
        Ok(Self {
            context,
            _js_runtime: js_runtime
        })
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
                LOCAL_RUNTIME.with(move |cell| {
                    cell.borrow()
                        .get_or_init(|| LocalRuntime::try_new(script).unwrap());
                });

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
        runtime.context.with(|ctx| {
            match event {
                Event::Request(req) => {
                    // NOTE: unwrap is safe here
                    // We receive a `None` only if the name of the function is more than the set
                    // kMaxLength. kMaxLength is set to a very high value ~ 1 Billion, so we
                    // don't expect to hit this limit.
                    let fn_name = name.into_js(&ctx).unwrap(); //  TODO: Check if this unwrap fails

                    let fn_value: Value = ctx.globals()
                        .get(fn_name)
                        .map_err(|_| anyhow::anyhow!("globalThis not initialized"))?;

                    let fn_server_emit = fn_value.as_function().unwrap(); //  TODO: Check if this unwrap fails
                    let args = prepare_args(&ctx, req)?;
                    let command: Option<Value> = fn_server_emit.call(args).ok();
                    command
                        .map(|output| Command::from_js(&ctx, output))
                        .transpose()
                        .map_err(|_| anyhow::anyhow!("deserialize failed")) //  TODO: Cast original error into anyhow
                }
            }
        })
    })
}
