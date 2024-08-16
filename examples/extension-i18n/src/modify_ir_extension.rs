use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use tailcall::core::blueprint::{ExtensionTrait, PrepareContext, ProcessContext};
use tailcall::core::config::KeyValue;
use tailcall::core::helpers::headers::to_mustache_headers;
use tailcall::core::valid::Validator;

#[derive(Clone, Debug)]
pub struct ModifyIrExtension {
    pub load_counter: Arc<Mutex<i32>>,
    pub prepare_counter: Arc<Mutex<i32>>,
    pub process_counter: Arc<Mutex<i32>>,
}

impl Default for ModifyIrExtension {
    fn default() -> Self {
        Self {
            load_counter: Arc::new(Mutex::new(0)),
            prepare_counter: Arc::new(Mutex::new(0)),
            process_counter: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl ExtensionTrait<ConstValue> for ModifyIrExtension {
    fn load(&self) {
        *(self.load_counter.lock().unwrap()) += 1;
    }

    async fn prepare(
        &self,
        context: PrepareContext<ConstValue>,
    ) -> Box<tailcall::core::ir::model::IR> {
        *(self.prepare_counter.lock().unwrap()) += 1;
        if let tailcall::core::ir::model::IR::IO(tailcall::core::ir::model::IO::Http {
            req_template,
            group_by,
            dl_id,
            http_filter,
        }) = *context.ir
        {
            let mut req_template = req_template;
            let headers = to_mustache_headers(&[KeyValue {
                key: "Authorization".to_string(),
                value: "Bearer 1234".to_string(),
            }]);

            match headers.to_result() {
                Ok(mut headers) => {
                    req_template.headers.append(&mut headers);
                }
                Err(_) => panic!("Headers are not structured properly"),
            };

            let ir = tailcall::core::ir::model::IR::IO(tailcall::core::ir::model::IO::Http {
                group_by,
                dl_id,
                http_filter,
                req_template,
            });
            Box::new(ir)
        } else {
            context.ir
        }
    }

    async fn process(
        &self,
        context: ProcessContext<ConstValue>,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        *(self.process_counter.lock().unwrap()) += 1;
        Ok(context.value)
    }
}
