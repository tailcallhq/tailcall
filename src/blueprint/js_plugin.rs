use std::{cell::RefCell, path::PathBuf};

use async_graphql_value::ConstValue;
use anyhow::Result;
use libloading::{library_filename, Library};
use tokio::{
  sync::{mpsc, oneshot},
  task::LocalSet,
};

use js_executor_interface::JsExecutor;

type CreateExecutor = fn(source: &str, with_input: bool) -> Box<dyn JsExecutor>;
type ChannelMessage = (oneshot::Sender<ConstValue>, ConstValue);

#[derive(Clone)]
pub struct JsPluginExecutor {
  sender: mpsc::UnboundedSender<ChannelMessage>,
  source: String,
}

impl JsPluginExecutor {
  pub fn source(&self) -> &str {
    &self.source
  }

  pub async fn call(&self, input: ConstValue) -> Result<ConstValue> {
    let (tx, rx) = oneshot::channel::<ConstValue>();

    self.sender.send((tx, input))?;

    Ok(rx.await?)
  }
}

pub struct JsPluginWrapper {
  library: Library,
  executors: RefCell<Vec<(mpsc::UnboundedReceiver<ChannelMessage>, String, bool)>>,
}

impl JsPluginWrapper {
  pub fn new(src: &str) -> Result<Self> {
    // TODO: figure out proper usage of src and relative directory for it
    let mut path = PathBuf::from(src);
    path.push(library_filename("js_executor"));

    let library = unsafe {
      let library = Library::new(&path)?;

      library
    };

    Ok(Self { library, executors: RefCell::default() })
  }

  pub fn start(self) -> Result<()> {
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

      let create_executor = unsafe { self.library.get::<CreateExecutor>(b"create_executor").expect("create_executor symbol should be defined in plugin") };

      for (mut receiver, script, with_input) in executors {
        let executor = create_executor(&script, with_input);

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

  pub fn create_executor(&self, source: String, with_input: bool) -> JsPluginExecutor {
    let (sender, receiver) = mpsc::unbounded_channel::<ChannelMessage>();

    self.executors.borrow_mut().push((receiver, source.clone(), with_input));

    JsPluginExecutor { sender, source }
  }
}
