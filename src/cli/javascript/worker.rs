use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use deno_core::serde_v8;
use deno_core::v8::{self, Function, Global, Local, Object, Value};
use deno_core::{FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions};
use tokio::sync::{oneshot, mpsc};
use tokio::task::spawn_local;
use tokio::time::{timeout_at, Instant};

use super::channel::CallbackMessage;
use super::{JsRequest, JsResponse};
use crate::blueprint;
use crate::cli::javascript::channel::Message;
use crate::cli::javascript::extensions::{console, fetch, timer_promises};

pub struct Worker {
    js_runtime: JsRuntime,
    function: Global<Function>,
}

// TODO: remove unwraps and handle errors
impl Worker {
    pub async fn new(
        script: blueprint::Script,
        http_sender: mpsc::UnboundedSender<CallbackMessage<JsRequest, JsResponse>>,
    ) -> anyhow::Result<Self> {
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![
                console::init_ops_and_esm(),
                timer_promises::init_ops_and_esm(),
                fetch::init_ops_and_esm(),
            ],
            ..Default::default()
        });

        js_runtime.op_state().borrow_mut().put(http_sender);

        let value = {
            let value = js_runtime.lazy_load_es_module_from_code(
                "file:///anon.js",
                FastString::from(script.source),
            )?;
            let scope = &mut js_runtime.handle_scope();
            let namespace = Local::new(scope, value);
            let namespace: Local<Object> = namespace.try_into()?;
            let key = v8::String::new(scope, "onEvent").unwrap();
            let value = namespace
                .get(scope, key.into())
                .ok_or(anyhow!("onEvent not found"))?;
            let value: Local<Function> = value.try_into()?;
            Global::new(scope, value)
        };
        log::debug!("JS Runtime created: {:?}", thread::current().name());
        Ok(Self { function: value, js_runtime })
    }

    pub async fn listen(mut self, work_receiver: loole::Receiver<CallbackMessage<Message, Message>>) -> Result<()> {
        let (result_tx, mut result_rx) = mpsc::unbounded_channel::<CallbackMessage<Result<Global<Value>>, Message>>();
        let mut event_loop_has_tasks = false;
        loop {
            // runs js event-loop with the ability to stop it
            // to react to external or internal events
            tokio::select! {
                // use biased mode to prioritize workload by its appearance below
                biased;
                // wait until the response from js is ready
                // to convert the value back to rust we need to use &mut js_runtime
                // that is also required for run_event_loop, so here we're
                // stopping event-loop and handle the response value
                Some((send_work_response, result)) = result_rx.recv() => {
                    let scope = &mut self.js_runtime.handle_scope();
                    let value = Local::new(scope, result.unwrap());
                    let message: Message = serde_v8::from_v8(scope, value).unwrap();

                    // ignore error for send since it would only happen
                    // when receiving channel is closed i.e. no one
                    // waits for the response and we may just drop it
                    let _ = send_work_response.send(message);
                },
                // accept new work for execute inside the script
                Ok((send_work_response, message)) = work_receiver.recv_async() => {
                    event_loop_has_tasks = true;
                    self.handle(message, send_work_response, result_tx.clone()).unwrap();
                },
                // run the js event-loop itself that will execute the script code.
                // do it only when we have tasks since otherwise that call will return
                // immediately and we will just burn cpu-cycles for calling this in outer loop
                _ = self.js_runtime.run_event_loop(PollEventLoopOptions::default()), if event_loop_has_tasks => {
                    // if this call is finished that means all the calls are executed
                    // and we need only wait for more work
                    event_loop_has_tasks = false;
                },

            }
        }
    }

    pub fn handle(
        &mut self,
        message: Message,
        send_work_response: oneshot::Sender<Message>,
        sender: mpsc::UnboundedSender<CallbackMessage<Result<Global<Value>>, Message>>,
    ) -> Result<()> {
        let message = {
            let mut scope = self.js_runtime.handle_scope();
            let message = serde_v8::to_v8(&mut scope, message)?;

            Global::new(&mut scope, message)
        };

        let call = self.js_runtime.call_with_args(&self.function, &[message]);
        // TODO: specify from config
        let call = timeout_at(Instant::now() + Duration::from_millis(1000), call);

        spawn_local(async move {
            let result = match call.await {
                Ok(result) => result,
                Err(_) => Err(anyhow!("Script timeout")),
            };

            sender.send((send_work_response, result)).unwrap()
        });

        Ok(())
    }
}
