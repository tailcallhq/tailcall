use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use tailcall::core::blueprint::{ExtensionTrait, PrepareContext, ProcessContext};

#[derive(Clone, Debug)]
pub struct TranslateExtension {
    pub load_counter: Arc<Mutex<i32>>,
    pub prepare_counter: Arc<Mutex<i32>>,
    pub process_counter: Arc<Mutex<i32>>,
}

impl Default for TranslateExtension {
    fn default() -> Self {
        Self {
            load_counter: Arc::new(Mutex::new(0)),
            prepare_counter: Arc::new(Mutex::new(0)),
            process_counter: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl ExtensionTrait<ConstValue> for TranslateExtension {
    fn load(&self) {
        *(self.load_counter.lock().unwrap()) += 1;
    }

    async fn prepare(
        &self,
        context: PrepareContext<ConstValue>,
    ) -> Box<tailcall::core::ir::model::IR> {
        *(self.prepare_counter.lock().unwrap()) += 1;
        context.ir
    }

    async fn process(
        &self,
        context: ProcessContext<ConstValue>,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        *(self.process_counter.lock().unwrap()) += 1;
        if let ConstValue::String(value) = context.value {
            let new_value = match value.as_str() {
                "Multi-layered client-server neural-net" => {
                    "Red neuronal cliente-servidor multicapa".to_string()
                }
                "Leanne Graham" => "Leona Grahm".to_string(),
                _ => value.to_string(),
            };
            Ok(ConstValue::String(new_value))
        } else {
            Ok(context.value)
        }
    }
}
