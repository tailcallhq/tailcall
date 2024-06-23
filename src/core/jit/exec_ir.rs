use serde_json_borrow::Value;

use crate::core::data_loader::DedupeResult;
use crate::core::ir::model::{IoId, IR};
use crate::core::ir::Error;
use crate::core::runtime::TargetRuntime;

/// An async executor for the IR.
struct Exec<'a> {
    runtime: TargetRuntime,
    store: DedupeResult<IoId, Value<'a>, Error>,
}

impl<'a> Exec<'a> {
    pub fn new(runtime: TargetRuntime) -> Self {
        Self { runtime, store: DedupeResult::new(true) }
    }

    pub fn execute(&self, ir: IR, value: Option<Value<'a>>) -> Result<Value<'a>, Error> {
        match ir {
            IR::Context(_) => todo!(),
            IR::Dynamic(_) => todo!(),
            IR::IO(_) => todo!(),
            IR::Cache(_) => todo!(),
            IR::Path(_, _) => todo!(),
            IR::Protect(_) => todo!(),
            IR::Map(_) => todo!(),
        }
    }
}
