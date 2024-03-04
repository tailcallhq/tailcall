use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use rhai::{Scope, AST};

use crate::cli::rhai_script::http_filter::{Command, Event};
use crate::{blueprint, WorkerIO};

pub struct ScriptMiddleware {
    engine: Rc<rhai::Engine>,
    ast: AST,
    scope: Scope<'static>,
}

impl ScriptMiddleware {
    pub fn try_new(script: String) -> anyhow::Result<Self> {
        let mut engine = rhai::Engine::new();

        engine
            .register_type_with_name::<Event>("Event")
            .register_fn("request", Event::get_request);

        engine
            .register_type_with_name::<Command>("Command")
            .register_fn("request", Command::new_request);
        let ast = engine.compile(script)?;
        Ok(Self { engine: Rc::new(engine), ast, scope: Scope::new() })
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

thread_local! {
  static LOCAL_RUNTIME: RefCell<OnceCell<ScriptMiddleware>> = const { RefCell::new(OnceCell::new()) };
}

lazy_static::lazy_static! {
    static ref CHANNEL_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .build()
        .expect("RHAI runtime not initialized");
}
#[async_trait::async_trait]
impl WorkerIO<Event, Command> for Runtime {
    async fn call(&self, name: String, event: Event) -> anyhow::Result<Option<Command>> {
        let script = self.script.clone();
        CHANNEL_RUNTIME
            .spawn(async move {
                LOCAL_RUNTIME.with(move |cell| {
                    cell.borrow()
                        .get_or_init(|| ScriptMiddleware::try_new(script.source).unwrap());
                });

                call(name, event)
            })
            .await?
    }
}

fn call(name: String, message: Event) -> anyhow::Result<Option<Command>> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let local_runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("rhai_script runtime not initialized"))?;
        let engine = &local_runtime.engine;
        engine
            .call_fn::<Command>(
                &mut local_runtime.scope,
                &local_runtime.ast,
                name,
                (message,),
            )
            .map(Some)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    })
}
