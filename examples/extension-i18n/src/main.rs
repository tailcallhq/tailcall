use std::sync::Arc;

use dotenvy::dotenv;
pub use modify_ir_extension::ModifyIrExtension;
use tailcall::cli::runtime;
use tailcall::cli::server::Server;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::reader::ConfigReader;
pub use translate_extension::TranslateExtension;

mod modify_ir_extension;
mod translate_extension;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let translate_ext = Arc::new(TranslateExtension::default());
    let modify_ir_ext = Arc::new(ModifyIrExtension::default());
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
    use hyper::{Body, Request};
    use serde_json::json;
    use tailcall::core::app_context::AppContext;
    use tailcall::core::async_graphql_hyper::GraphQLRequest;
    use tailcall::core::http::{handle_request, Response};
    use tailcall::core::rest::EndpointSet;
    use tailcall::core::HttpIO;

    use super::*;

    struct MockHttp;

    #[async_trait::async_trait]
    impl HttpIO for MockHttp {
        async fn execute(
            &self,
            _request: reqwest::Request,
        ) -> anyhow::Result<Response<hyper::body::Bytes>> {
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
            })
            .to_string();
            Ok(Response {
                body: data.into(),
                status: reqwest::StatusCode::OK,
                headers: reqwest::header::HeaderMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_tailcall_extensions() {
        let translate_ext = Arc::new(TranslateExtension::default());
        let modify_ir_ext = Arc::new(ModifyIrExtension::default());
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
            .insert("translate".to_string(), translate_ext.clone());
        extensions
            .plugin_extensions
            .insert("modify_ir".to_string(), modify_ir_ext.clone());
        let config_module = config_module.merge_extensions(extensions);
        let blueprint = Blueprint::try_from(&config_module).unwrap();
        let app_context = AppContext::new(blueprint, runtime, EndpointSet::default());

        let query = json!({
            "query": "{ user(id: 1) { id name company { catchPhrase } } }"
        });
        let body = Body::from(query.to_string());
        let req = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:8800/graphql")
            .body(body)
            .unwrap();

        let response = handle_request::<GraphQLRequest>(req, Arc::new(app_context))
            .await
            .unwrap();
        let response = tailcall::core::http::Response::from_hyper(response)
            .await
            .unwrap();

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
            response.body,
            hyper::body::Bytes::from(expected_response.to_string()),
            "Unexpected response from server"
        );

        assert_eq!(translate_ext.load_counter.lock().unwrap().to_owned(), 2);
        assert_eq!(translate_ext.process_counter.lock().unwrap().to_owned(), 2);
        assert_eq!(translate_ext.prepare_counter.lock().unwrap().to_owned(), 2);

        assert_eq!(modify_ir_ext.load_counter.lock().unwrap().to_owned(), 1);
        assert_eq!(modify_ir_ext.process_counter.lock().unwrap().to_owned(), 1);
        assert_eq!(modify_ir_ext.prepare_counter.lock().unwrap().to_owned(), 1);
    }
}
