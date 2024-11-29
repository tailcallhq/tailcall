use serde_json::Value;
use tailcall_valid::Valid;

use crate::core::blueprint::{BlueprintError, DynamicValue};
use crate::core::ir::model::IR;

pub fn apply_select(input: (IR, &Option<Value>)) -> Valid<IR, BlueprintError> {
    let (mut ir, select) = input;

    if let Some(select_value) = select {
        let dynamic_value = match DynamicValue::try_from(select_value) {
            Ok(dynamic_value) => dynamic_value.prepend("args"),
            Err(e) => {
                return Valid::fail_with(
                    BlueprintError::SyntaxErrorWhenParsing(format!("{:?}", select)),
                    BlueprintError::Error(e),
                )
            }
        };

        ir = ir.pipe(IR::Dynamic(dynamic_value));
        Valid::succeed(ir)
    } else {
        Valid::succeed(ir)
    }
}
