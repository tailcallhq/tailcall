use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use libloading::{library_filename, Library, Symbol};

use js_executor_interface::JsExecutor;

#[derive(Clone, Debug)]
pub struct JsPluginWrapper {
  library: Arc<Library>,
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

    Ok(Self { library: Arc::new(library) })
  }

  pub fn eval(&self, source: &str, input: &str) -> Result<async_graphql::Value> {
    let executor = unsafe {
      let executor: Symbol<JsExecutor> = self.library.get(b"eval")?;

      executor
    };

    let result = executor(source, input).unwrap();

    Ok(serde_json::from_str(result.as_str())?)
  }
}
