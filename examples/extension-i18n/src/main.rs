use std::sync::Arc;

use async_graphql_value::ConstValue;
use dotenvy::dotenv;
use tailcall::cli::runtime;
use tailcall::cli::server::Server;
use tailcall::core::blueprint::{Blueprint, ExtensionLoader};
use tailcall::core::config::reader::ConfigReader;

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
        // TODO: implement function
        // TODO: figure out why this is not executing
        println!("params: {:?}", params);
        println!("ir: {:?}", ir);
        ir
    }

    fn process(
        &self,
        params: ConstValue,
        value: ConstValue,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        // TODO: implement function
        println!("params: {:?}", params);
        println!("value: {:?}", value);
        Ok(value)
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
        // TODO: implement function
        println!("params: {:?}", params);
        println!("ir: {:?}", ir);
        ir
    }

    fn process(
        &self,
        params: ConstValue,
        value: ConstValue,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
        // TODO: implement function
        println!("params: {:?}", params);
        println!("value: {:?}", value);
        Ok(value)
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
