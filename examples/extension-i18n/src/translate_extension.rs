use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use serde_json::Value;
use tailcall::core::blueprint::{ExtensionTrait, PrepareContext, ProcessContext};

#[derive(Clone, Debug)]
pub struct TranslateExtension {
    pub load_counter: Arc<Mutex<i32>>,
    pub prepare_counter: Arc<Mutex<i32>>,
    pub process_counter: Arc<Mutex<i32>>,
    pub translations: Arc<Value>,
}

impl Default for TranslateExtension {
    fn default() -> Self {
        let file = File::open("./src/translations.json").unwrap();
        let reader = BufReader::new(file);

        let translations = Arc::new(serde_json::from_reader(reader).unwrap());
        Self {
            load_counter: Arc::new(Mutex::new(0)),
            prepare_counter: Arc::new(Mutex::new(0)),
            process_counter: Arc::new(Mutex::new(0)),
            translations,
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
            if let Some(new_value) = self.translations.get(&value) {
                Ok(ConstValue::String(new_value.as_str().unwrap().to_string()))
            } else {
                Ok(ConstValue::String(value))
            }
        } else {
            Ok(context.value)
        }
    }
}
