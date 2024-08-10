use std::sync::Arc;

use async_graphql_value::ConstValue;
use dotenvy::dotenv;
use futures::executor::block_on;
use tailcall::cli::runtime;
use tailcall::cli::server::Server;
use tailcall::core::blueprint::{Blueprint, ExtensionLoader};
use tailcall::core::config::reader::ConfigReader;
use tailcall::core::config::KeyValue;
use tailcall::core::helpers::headers::to_mustache_headers;
use tailcall::core::valid::Validator;

#[derive(Clone, Debug)]
pub struct TranslateExtension;

impl ExtensionLoader for TranslateExtension {
    fn load(&self) {
        println!("TranslateExtension...loaded!")
    }

    fn prepare(
        &self,
        ir: Box<tailcall::core::ir::model::IR>,
        params: ConstValue,
    ) -> Box<tailcall::core::ir::model::IR> {
        println!("params: {:?}", params);
        println!("ir: {:?}", ir);
        ir
    }

    fn process(
        &self,
        params: ConstValue,
        value: ConstValue,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        println!("params: {:?}", params);
        println!("value: {:?}", value);
        if let ConstValue::String(value) = value {
            let new_value = block_on(translate(&value));
            Ok(ConstValue::String(new_value))
        } else {
            Ok(value)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModifyIrExtension;

impl ExtensionLoader for ModifyIrExtension {
    fn load(&self) {
        println!("ModifyIrExtension...loaded!")
    }

    fn prepare(
        &self,
        ir: Box<tailcall::core::ir::model::IR>,
        params: ConstValue,
    ) -> Box<tailcall::core::ir::model::IR> {
        println!("params: {:?}", params);
        println!("ir: {:?}", ir);

        if let tailcall::core::ir::model::IR::IO(tailcall::core::ir::model::IO::Http {
            req_template,
            group_by,
            dl_id,
            http_filter,
        }) = *ir
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
            ir
        }
    }

    fn process(
        &self,
        params: ConstValue,
        value: ConstValue,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        println!("params: {:?}", params);
        println!("value: {:?}", value);
        Ok(value)
    }
}

async fn translate(value: &str) -> String {
    match value {
        "Multi-layered client-server neural-net" => {
            "Red neuronal cliente-servidor multicapa".to_string()
        }
        "Leanne Graham" => "Leona Grahm".to_string(),
        _ => value.to_string(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Extensions Example");
    let translate_ext = Arc::new(TranslateExtension {});
    let modify_ir_ext = Arc::new(ModifyIrExtension {});
    if let Ok(path) = dotenv() {
        tracing::info!("Env file: {:?} loaded", path);
    }
    let runtime = runtime::init(&Blueprint::default());
    let config_reader = ConfigReader::init(runtime.clone());
    let file_paths = ["./examples/extension-i18n/main.graphql"];
    let config_module = config_reader.read_all(file_paths.as_ref()).await?;
    let mut extensions = config_module.extensions().clone();
    extensions
        .plugin_extensions
        .insert("translate".to_string(), translate_ext);
    extensions
        .plugin_extensions
        .insert("modify_ir".to_string(), modify_ir_ext);
    let config_module = config_module.merge_extensions(extensions);
    let server = Server::new(config_module);
    server.fork_start().await?;
    Ok(())
}
