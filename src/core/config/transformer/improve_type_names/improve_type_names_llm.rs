use std::collections::HashMap;

use reqwest;
use reqwest::Error;

use crate::core::config::Config;

pub struct ImproveTypeNamesLLM;

type OriginalTypeName = String;
type AIGeneratedTypeName = String;

struct LLMRequest {
    original_type_name: OriginalTypeName,
    prompt: String,
}

struct LLMResponse {
    original_type_name: OriginalTypeName,
    suggested_type_names: Vec<AIGeneratedTypeName>,
}

impl ImproveTypeNamesLLM {
    pub fn generate_llm_type_names(
        config: Config,
    ) -> Result<HashMap<OriginalTypeName, Vec<AIGeneratedTypeName>>, String> {
        let mut llm_requests: Vec<LLMRequest> = Vec::new();

        for (type_name, type_info) in config.types.iter() {
            if config.is_root_operation_type(type_name) {
                continue;
            }

            let mut fields: Vec<String> = Vec::new();
            for (field_name, field_info) in type_info.fields.iter() {
                fields.push(format!("{}: {}", field_name, field_info.type_of));
                //ex: T2: {name: String, age: Int}
            }

            let prompt = Self::generate_llm_request(type_name, fields);
            llm_requests.push(prompt);
        }

        #[cfg(not(target_arch = "wasm32"))]
        return tokio::task::block_in_place(move || Self::make_llm_requests(llm_requests));
        #[cfg(target_arch = "wasm32")]
        Self::make_llm_requests(llm_requests)
    }

    fn generate_llm_request(name: &str, fields: Vec<String>) -> LLMRequest {
        let mut type_prompt = format!("{} {{", name);

        for (index, field) in fields.iter().enumerate() {
            type_prompt.push_str(field);
            if index != fields.len() - 1 {
                type_prompt.push(' ');
            }
        }
        type_prompt.push('}');
        let prompt = format!("{{\"prompt\":\"{}\"}}", type_prompt);
        LLMRequest { original_type_name: name.to_string(), prompt }
    }

    fn make_llm_requests(
        llm_requests: Vec<LLMRequest>,
    ) -> Result<HashMap<OriginalTypeName, Vec<AIGeneratedTypeName>>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let client = reqwest::Client::new();

            let mut results: HashMap<OriginalTypeName, Vec<AIGeneratedTypeName>> = HashMap::new();
            for llm_request in llm_requests.into_iter() {
                let client = client.clone();
                let response = Self::get_llm_response(client, llm_request).await;
                match response {
                    Ok(result) => {
                        results.insert(result.original_type_name, result.suggested_type_names);
                    }
                    Err(e) => return Err(e.to_string()),
                };
            }

            Ok(results)
        })
    }

    async fn get_llm_response(
        client: reqwest::Client,
        llm_request: LLMRequest,
    ) -> Result<LLMResponse, Error> {
        let response = client
            .post("https://ec56-2405-201-101e-60f3-fc34-439-491b-fe01.ngrok-free.app/type_name")
            .header("Content-Type", "application/json")
            .header(reqwest::header::TRANSFER_ENCODING, "chunked")
            .body(llm_request.prompt)
            .send()
            .await?;

        let result = response.text().await?;
        let result = result.replace([' ', '\n', '[', ']', '"'], ""); //ex: [User,Profile,Contact,Person,Entity]
        let result = result.split(',').map(|s| s.to_string()).collect();
        Ok(LLMResponse {
            original_type_name: llm_request.original_type_name.to_string(),
            suggested_type_names: result,
        })
    }
}
