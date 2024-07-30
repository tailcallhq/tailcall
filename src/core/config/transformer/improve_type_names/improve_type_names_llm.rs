use std::collections::HashMap;

use reqwest;
use reqwest::Error;
use tokio::task::JoinSet;

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
            if type_name == "Query" || type_name == "Mutation" || type_name == "Subscription" {
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
        let run_time = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        run_time.block_on(async {
            let client = reqwest::Client::new();
            let mut tasks_set: JoinSet<Result<LLMResponse, Error>> = JoinSet::new();

            for llm_request in llm_requests.into_iter() {
                let client = client.clone();
                tasks_set.spawn(async move { Self::get_llm_response(client, llm_request).await });
            }

            let mut results: HashMap<OriginalTypeName, Vec<AIGeneratedTypeName>> = HashMap::new();
            while let Some(task) = tasks_set.join_next().await {
                match task {
                    Ok(response) => {
                        match response {
                            Ok(result) => results
                                .insert(result.original_type_name, result.suggested_type_names),
                            Err(e) => return Err(e.to_string()),
                        };
                    }
                    Err(err) => return Err(err.to_string()),
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
            .post("https://e8b2-2405-201-101e-60f3-e984-6b14-8703-e5ed.ngrok-free.app/type_name")
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

#[cfg(test)]
mod test {
    use std::fs;

    use tailcall_fixtures::configs;

    use crate::core::config::Config;
    use crate::core::valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_llm_type_name_generator_transform() {
        let config = Config::from_sdl(read_fixture(configs::AUTO_GENERATE_CONFIG).as_str())
            .to_result()
            .unwrap();

        let llm_response =
            super::ImproveTypeNamesLLM::generate_llm_type_names(config.clone()).unwrap();
        insta::assert_snapshot!(format!("{:?}", llm_response));
    }

    #[test]
    fn test_llm_type_name_generator_with_cyclic_types() -> anyhow::Result<()> {
        let config = Config::from_sdl(read_fixture(configs::CYCLIC_CONFIG).as_str())
            .to_result()
            .unwrap();

        let llm_response =
            super::ImproveTypeNamesLLM::generate_llm_type_names(config.clone()).unwrap();
        insta::assert_snapshot!(format!("{:?}", llm_response));

        Ok(())
    }

    #[test]
    fn test_llm_type_name_generator() -> anyhow::Result<()> {
        let config = Config::from_sdl(read_fixture(configs::NAME_GENERATION).as_str())
            .to_result()
            .unwrap();

        let llm_response =
            super::ImproveTypeNamesLLM::generate_llm_type_names(config.clone()).unwrap();
        insta::assert_snapshot!(format!("{:?}", llm_response));

        Ok(())
    }
}
