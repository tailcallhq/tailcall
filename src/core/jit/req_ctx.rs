use serde_json_borrow::Value;

use crate::core::data_loader::DedupeResult;
use crate::core::ir::Error;
use crate::core::ir::model::IoId;

pub struct RequestContext<'a> {
    pub cache: DedupeResult<IoId, Value<'a>, Error>,
    pub dedupe_handler: DedupeResult<IoId, Value<'a>, Error>,
}