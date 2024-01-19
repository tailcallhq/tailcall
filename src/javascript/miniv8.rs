use std::cell::RefCell;

use anyhow::Result;
use async_graphql_value::ConstValue;
use js_executor::JsExecutor;
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;

use super::{JsPluginExecutorInterface, JsPluginWrapperInterface};

type ChannelMessage = (oneshot::Sender<ConstValue>, ConstValue);

#[derive(Clone, Debug)]
pub struct JsPluginExecutor {
  sender: mpsc::UnboundedSender<ChannelMessage>,
  source: String,
}

impl JsPluginExecutorInterface for JsPluginExecutor {
  fn source(&self) -> &str {
    &self.source
  }

  async fn call(&self, input: ConstValue) -> Result<ConstValue> {
    let (tx, rx) = oneshot::channel::<ConstValue>();

    self.sender.send((tx, input))?;

    Ok(rx.await?)
  }
}

pub struct JsPluginWrapper {
  executors: RefCell<Vec<(mpsc::UnboundedReceiver<ChannelMessage>, String, bool)>>,
}

impl JsPluginWrapperInterface<JsPluginExecutor> for JsPluginWrapper {
  fn try_new() -> Result<Self> {
    Ok(Self { executors: RefCell::default() })
  }

  fn start(self) -> Result<()> {
    let executors = self.executors.take();

    if executors.is_empty() {
      return Ok(());
    }

    std::thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
      let local = LocalSet::new();

      for (mut receiver, script, with_input) in executors {
        let executor = JsExecutor::new(&script, with_input);

        local.spawn_local(async move {
          while let Some((response, input)) = receiver.recv().await {
            let result = executor.eval(input);

            response.send(result.unwrap()).unwrap();
          }
        });
      }

      rt.block_on(local);
    });

    Ok(())
  }

  fn create_executor(&self, source: String, with_input: bool) -> JsPluginExecutor {
    let (sender, receiver) = mpsc::unbounded_channel::<ChannelMessage>();

    self.executors.borrow_mut().push((receiver, source.clone(), with_input));

    JsPluginExecutor { sender, source }
  }
}
