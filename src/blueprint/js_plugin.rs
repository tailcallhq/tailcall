use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use libloading::{library_filename, Library, Symbol};

use js_executor_interface::JsExecutor;

#[derive(Clone)]
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

  pub fn create_executor(&self, source: String) -> Result<Box<dyn JsExecutor>> {
    let create_executor: Symbol<fn(source: &str) -> Box<dyn JsExecutor>> =
      unsafe { self.library.get(b"create_executor")? };

    Ok(create_executor(&source))
  }
}
