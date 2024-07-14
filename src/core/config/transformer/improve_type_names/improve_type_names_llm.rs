use reqwest;
use tokio::task::JoinSet;
use serde::Deserialize;
use std::borrow::Cow;
use reqwest::Error;
use crate::core::config::Config;
use crate::core::valid::Valid;
use crate::core::transform::Transform;

#[derive(Deserialize, Debug)]
pub struct OLLAMAResponse<'a> {
    #[serde(borrow)]
    response: Cow<'a, [u8]>,
    done: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LLMImprovedNamesSuggestion {
    original_type_name: String,
    suggested_type_names: Vec<String>,
}


pub struct ImproveTypeNamesLLM;

const LLM_SYSTEM: &str = r#"Given the GraphQL type definition below, provide a response in the form of a JSONP callback. The function should be named \"callback\" and should return JSON suggesting at least five suitable alternative names for the type. Each suggested name should be concise, preferably a single word, and capture the essence of the data it represents based on the roles and relationships implied by the field names. \n\n```graphql\ntype T {\n  name: String,\n  age: Int,\n  website: String\n}\n```\n\n**Expected JSONP Format:**\n\n```javascript\ncallback({\n  \"originalTypeName\": \"T\",\n  \"suggestedTypeNames\": [\"Person\",\"Profile\",\"Member\",\"Individual\",\"Contact\"\n  ]\n});\n``` provide the suggestions without explaining what you've done."#;

impl ImproveTypeNamesLLM {
    fn generate_type_names(config: Config) -> Config {
        let mut llm_requests: Vec<String> = Vec::new();

        for (type_name, type_info) in config.types.iter() {
            if type_name == "Query" || type_name == "Mutation" || type_name == "Subscription" {
                continue;
            }

            println!("{:?}", type_name);
            let mut fields: Vec<String> = Vec::new();
            for (field_name, field_info) in type_info.fields.iter() {
                fields.push(format!("{}: {}", field_name, field_info.type_of));
            }

            let prompt = Self::generate_type_prompt(type_name, fields);
            llm_requests.push(Self::generate_llm_request_body(prompt));
        }

        // This function will block untill the llm respond with all the new types names
        let x= Self::make_llm_requests(llm_requests);

        dbg!("{}", x);

        config
    }

    fn generate_type_prompt(name: &str, fields: Vec<String>) -> String {
        let mut type_prompt = format!("type {} {{", name);

        for (index, field) in fields.iter().enumerate() {
            type_prompt.push_str(field);
            if index != fields.len() - 1 {
                type_prompt.push_str(",");
            }
        }

        type_prompt
    }

    fn generate_llm_request_body(prompt: String) -> String {
        format!(
            r#"{{ "model": "llama3", "format": "json", "system": "{}",  "prompt": "{}" }}"#,
            LLM_SYSTEM, prompt
        )
    }

    fn make_llm_requests(llm_requests: Vec<String>) -> Valid<Vec<String>, String> {
        let run_time = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        run_time.block_on(async {
            let client = reqwest::Client::new();
            let mut tasks_set: JoinSet<Result<Vec<u8>, Error>> = JoinSet::new();

            for llm_request in llm_requests.into_iter() {
                let client = client.clone();
                tasks_set.spawn(async move {
                    Self::get_llm_callback(client, llm_request).await
                }); 
            }

            let mut callbacks: Vec<String> = vec![];
            while let Some(task) = tasks_set.join_next().await {
                match task {
                    Ok(callback) => {
                        match callback {
                            Ok(callback_bytes) => callbacks.push(std::str::from_utf8(&callback_bytes).unwrap().to_string()),
                            Err(e) => return Valid::fail(e.to_string()) 
                        };
                    },
                    Err(err) => return Valid::fail(err.to_string())
                };
            }

            return Valid::succeed(callbacks);
        })
    }

    // This function will return a bytes vector of callbacks that were AI generated
    // callback({ "originalTypeName": "T", "suggestedTypeNames": ["Post"] });
    async fn get_llm_callback(client: reqwest::Client, llm_request: String) -> Result<Vec<u8>, Error> {
        println!("{:?}", llm_request);
        let mut response = client
            .post("https://d4f2-35-221-237-178.ngrok-free.app/api/generate")
            .header("Content-Type", "application/json")
            .header(reqwest::header::TRANSFER_ENCODING, "chunked")
            .body(llm_request)
            .send()
            .await?;

        let mut should_build_callback_str = false;
        let mut callback: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            let slice: &[u8] = &chunk;
            match serde_json::from_slice::<OLLAMAResponse>(slice) {
                Ok(ollama_response) => {
                    if ollama_response.response == "callback".as_bytes() {
                        callback.extend_from_slice(&ollama_response.response);
                        should_build_callback_str = true;
                    } else if should_build_callback_str {
                        if ollama_response.response == "});".as_bytes() {
                            should_build_callback_str = false;
                        }
                        callback.extend_from_slice(&ollama_response.response);
                    }
                },
                Err(_) => {
                    println!("there was an issue parsing the request");
                }
            };
        }

        Ok(callback)
    }
}

impl Transform for ImproveTypeNamesLLM {
    type Value = Config;
    type Error = String;
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = Self::generate_type_names(config);

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use tailcall_fixtures::configs;
    use crate::core::config::Config;
    use crate::core::valid::Validator;

    use crate::core::transform::Transform;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_llm_stuff() {
        let config = Config::from_sdl(read_fixture(configs::AUTO_GENERATE_CONFIG).as_str())
            .to_result()
            .unwrap();

        super::ImproveTypeNamesLLM.transform(config).to_result().unwrap();
        assert_eq!(1, 2);
    }
}
