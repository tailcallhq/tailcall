use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::BlueprintError;
use crate::core::config::JS;
use crate::core::ir::model::{IO, IR};

pub struct CompileJs<'a> {
    pub js: &'a JS,
    pub script: &'a Option<String>,
}

pub fn compile_js(inputs: CompileJs) -> Valid<IR, BlueprintError> {
    let name = &inputs.js.name;
    Valid::from_option(inputs.script.as_ref(), BlueprintError::ScriptIsRequired)
        .map(|_| IR::IO(IO::Js { name: name.to_string() }))
}
