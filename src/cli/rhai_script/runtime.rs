use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use rhai::{Scope, AST};

use crate::cli::rhai_script::http_filter::Message;
use crate::{blueprint, WorkerIO};

pub struct ScriptMiddleware {
    engine: Rc<rhai::Engine>,
    ast: AST,
}

impl ScriptMiddleware {
    pub fn try_new(script: String) -> anyhow::Result<Self> {
        let engine = Rc::new(rhai::Engine::new());

        let ast = engine.compile(script);
        let ast = ast?;
        Ok(Self { engine, ast })
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

impl WorkerIO<Message, Message> for Runtime {
    fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        log::debug!("event: {:?}", event);
        let command = LOCAL_RUNTIME.with(move |cell| {
            let script = self.script.clone();
            cell.borrow().get_or_init(|| {
                ScriptMiddleware::try_new(script.source)
                    .expect("rhai_script runtime not initialized")
            });
            on_event(event)
        });
        command
    }
}

fn on_event(message: Message) -> anyhow::Result<Message> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let local_runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("rhai_script runtime not initialized"))?;
        let mut scope = Scope::new();
        let engine = &local_runtime.engine;

        engine
            .call_fn::<Message>(&mut scope, &local_runtime.ast, "onEvent", (message,))
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    })
}
