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
    }

    fn prepare(
        &self,
        ir: Box<tailcall::core::ir::model::IR>,
        params: ConstValue,
    ) -> Box<tailcall::core::ir::model::IR> {
        ir
    }

    fn process(
        &self,
        params: ConstValue,
        value: ConstValue,
    ) -> Result<ConstValue, tailcall::core::ir::Error> {
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
    }

    fn prepare(
        &self,
        ir: Box<tailcall::core::ir::model::IR>,
        params: ConstValue,
    ) -> Box<tailcall::core::ir::model::IR> {
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


#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;
    use tailcall::core::{http::Response, HttpIO};

    use super::*;

    struct MockHttp;

    #[async_trait::async_trait]
    impl HttpIO for MockHttp {
        async fn execute(&self, _request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>> {
            let data = json!({
                "id": 1,
                "name": "Leanne Graham",
                "username": "Bret",
                "email": "Sincere@april.biz",
                "address": {
                    "street": "Kulas Light",
                    "suite": "Apt. 556",
                    "city": "Gwenborough",
                    "zipcode": "92998-3874",
                    "geo": {
                            "lat": "-37.3159",
                            "lng": "81.1496"
                    }
                },
                "phone": "1-770-736-8031 x56442",
                "website": "hildegard.org",
                "company": {
                    "name": "Romaguera-Crona",
                    "catchPhrase": "Multi-layered client-server neural-net",
                    "bs": "harness real-time e-markets"
                }
            }).to_string();
            Ok(Response {
                body: data.into(),
                status: reqwest::StatusCode::OK,
                headers: reqwest::header::HeaderMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_tailcall_extensions() {
        let translate_ext = Arc::new(TranslateExtension {});
        let modify_ir_ext = Arc::new(ModifyIrExtension {});
        if let Ok(path) = dotenv() {
            tracing::info!("Env file: {:?} loaded", path);
        }
        let mut runtime = runtime::init(&Blueprint::default());
        runtime.http = Arc::new(MockHttp {});
        runtime.http2_only = Arc::new(MockHttp {});
        let config_reader = ConfigReader::init(runtime.clone());
        let file_paths = ["./main.graphql"];
        let config_module = config_reader.read_all(file_paths.as_ref()).await.unwrap();
        let mut extensions = config_module.extensions().clone();
        extensions
            .plugin_extensions
            .insert("translate".to_string(), translate_ext);
        extensions
            .plugin_extensions
            .insert("modify_ir".to_string(), modify_ir_ext);
        let config_module = config_module.merge_extensions(extensions);
        let mut server = Server::new(config_module);
        let url = "http://127.0.0.1:8800/graphql";
        let server_up_receiver = server.server_up_receiver();

        tokio::spawn(async move {
            server.start().await.unwrap();
        });

        server_up_receiver
            .await
            .expect("Server did not start up correctly");

        // required since our cert is self signed
        let client = Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let query = json!({
            "query": "{ user(id: 1) { id name company { catchPhrase } } }"
        });

        let client = client.clone();
        let url = url.to_owned();
        let query = query.clone();

        let response = tokio::spawn(async move {
            let response = client.post(url).json(&query).send().await;
            let response = response.unwrap();
            let response_body: serde_json::Value = response.json().await.expect("Request should success");
            response_body
        }).await
        .expect("Spawned task should success");

        let expected_response = json!({
            "data": {
                "user": {
                    "id": 1,
                    "name": "Leona Grahm",
                    "company": {
                        "catchPhrase": "Red neuronal cliente-servidor multicapa"
                    }
                }
            }
        });
        assert_eq!(
            response, expected_response,
            "Unexpected response from server"
        );
    }
}