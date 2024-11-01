use serde_json::Value;

use crate::core::blueprint::DynamicValue;
use crate::core::ir::model::IR;
use crate::core::valid::Valid;

pub fn apply_select(input: (IR, &Option<Value>)) -> Valid<IR, miette::MietteDiagnostic> {
    let (mut ir, select) = input;

    if let Some(select_value) = select {
        let dynamic_value = match DynamicValue::try_from(select_value) {
            Ok(dynamic_value) => dynamic_value.prepend("args"),
            Err(e) => {
                return Valid::fail(miette::diagnostic!(
                    help = format!("syntax error when parsing `{:?}`", select),
                    "{}",
                    e
                ))
            }
        };

        ir = ir.pipe(IR::Dynamic(dynamic_value));
        Valid::succeed(ir)
    } else {
        Valid::succeed(ir)
    }
}
