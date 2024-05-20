use convert_case::Casing;
use serde_json::Value;
use url::Url;

use crate::core::config::{Arg, Config, Field, Http, Type};
use crate::core::helpers::gql_type::{
    detect_gql_data_type, is_list_type, is_primitive, is_valid_field_name, to_gql_type,
};
use crate::core::merge_right::MergeRight;

#[derive(Debug)]
struct UrlQuery {
    key: String,
    data_type: String, // gql type.
}

#[derive(Debug)]
struct UrlQueryParser {
    queries: Vec<UrlQuery>,
}

impl UrlQueryParser {
    fn new(url: &Url) -> Self {
        let query_list: Vec<_> = url
            .query_pairs()
            .map(|(k, v)| UrlQuery { key: k.to_string(), data_type: detect_gql_data_type(&v) })
            .collect();
        Self { queries: query_list }
    }
}

struct ConfigGenerator {
    /// final configuration that's being built up.
    config: Config,
    /// Used to generate the type names.
    type_counter: i32,
}

impl ConfigGenerator {
    fn new() -> Self {
        Self { config: Config::default(), type_counter: 1 }
    }

    fn insert_type(&mut self, type_name: &str, actual_type: Type) {
        self.config.types.insert(type_name.to_string(), actual_type);
    }

    fn should_generate_type(&self, value: &Value) -> bool {
        match value {
            Value::Array(json_array) => !json_array.is_empty(),
            Value::Object(json_object) => {
                !json_object.is_empty()
                    && !json_object
                        .keys()
                        .any(|json_property| !is_valid_field_name(json_property))
            }
            _ => true, // generate for all primitive types.
        }
    }

    fn generate_scalar(&mut self) -> String {
        if self.config.types.contains_key("Any") {
            return "Any".to_string();
        }
        self.insert_type("Any", Type::default());
        "Any".to_string()
    }

    fn generate_types(&mut self, json_value: &Value) -> String {
        match json_value {
            Value::Array(json_arr) => {
                if let Some(json_item) = json_arr.first() {
                    return if is_primitive(json_item) {
                        to_gql_type(json_item)
                    } else {
                        self.generate_types(json_item)
                    };
                }
                // generate a scalar if array is empty.
                self.generate_scalar()
            }
            Value::Object(json_obj) => {
                let mut ty = Type::default();
                for (json_property, json_val) in json_obj {
                    if !self.should_generate_type(json_val) {
                        // if object, array is empty or object has in-compatible fields then
                        // generate scalar for it.
                        let field = Field {
                            type_of: self.generate_scalar(),
                            list: is_list_type(json_val),
                            ..Default::default()
                        };
                        ty.fields.insert(json_property.to_string(), field);
                        continue;
                    }

                    let mut field = Field::default();
                    if is_primitive(json_val) {
                        field.type_of = to_gql_type(json_val);
                    } else {
                        let type_name = self.generate_types(json_val);
                        field.type_of = type_name;
                        field.list = is_list_type(json_val);
                    }
                    ty.fields.insert(json_property.to_string(), field);
                }
                let type_name = format!("T{}", self.type_counter);
                self.type_counter += 1;
                self.insert_type(&type_name, ty);
                type_name
            }
            other => to_gql_type(other),
        }
    }

    fn generate_query_type(&mut self, url: &Url, value: &Value, root_type_name: String) {
        let mut field = Field {
            list: is_list_type(value),
            type_of: root_type_name.to_string(),
            ..Default::default()
        };

        let query_list = UrlQueryParser::new(url).queries;

        // collect queries to generate mustache format path.
        let mut path_queries: Vec<String> = Vec::with_capacity(query_list.len());

        // add args to field and prepare mustache template format queries.
        for query in query_list {
            path_queries.push(format!("{}={{{{.args.{}}}}}", query.key, query.key));

            let arg = Arg {
                list: query.data_type == "List",
                type_of: query.data_type,
                required: true,
                ..Default::default()
            };

            field.args.insert(query.key, arg);
        }

        // add path in http directive.
        let mut http = Http::default();
        let mut complete_path = url.path().to_string();
        if !path_queries.is_empty() {
            complete_path = format!("{}?{}", complete_path, path_queries.join("&"))
        }
        http.path = complete_path;
        field.http = Some(http);

        let mut ty = Type::default();
        ty.fields
            .insert(root_type_name.to_case(convert_case::Case::Camel), field);
        self.insert_type("Query", ty);
    }

    fn generate_upstream(&mut self, url: &Url) {
        self.config.upstream.base_url =
            Some(format!("{}://{}", url.scheme(), url.host_str().unwrap()));
    }

    fn generate_schema(&mut self) {
        self.config.schema.query = Some("Query".to_string());
    }

    fn generate(&mut self, url: &str, resp: &Value) -> anyhow::Result<()> {
        let url = Url::parse(url)?;
        let root_type_name = self.generate_types(resp);
        self.generate_query_type(&url, resp, root_type_name);
        self.generate_upstream(&url);
        self.generate_schema();
        Ok(())
    }
}

pub struct ConfigGenerationRequest<'a> {
    pub url: &'a str,
    resp: &'a Value,
}

impl<'a> ConfigGenerationRequest<'a> {
    pub fn new(url: &'a str, resp: &'a Value) -> Self {
        Self { url, resp }
    }
}

pub fn from_json(config_gen_req: &[ConfigGenerationRequest]) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let mut ctx = ConfigGenerator::new();
    for request in config_gen_req.iter() {
        ctx.generate(request.url, request.resp)?;
        config = config.merge_right(ctx.config.clone());
    }
    Ok(config)
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use url::Url;

    use crate::core::generator::from_json::{ConfigGenerator, UrlQueryParser};

    #[test]
    fn test_should_generate_type() {
        let config_gen = ConfigGenerator::new();
        assert!(config_gen.should_generate_type(&json!("Testing")));
        assert!(config_gen.should_generate_type(&json!(12)));
        assert!(config_gen.should_generate_type(&json!(12.3)));
        assert!(config_gen.should_generate_type(&json!(-12)));
        assert!(config_gen.should_generate_type(&json!(-12.2)));
        assert!(config_gen.should_generate_type(&json!(true)));
        assert!(config_gen.should_generate_type(&json!(false)));
        assert!(config_gen.should_generate_type(&json!([1, 2, 3])));
        assert!(config_gen.should_generate_type(&json!({"name":"test", "age": 12})));

        // ignore the empty types.
        assert!(!config_gen.should_generate_type(&json!([])));
        assert!(!config_gen.should_generate_type(&json!({})));

        // not valid field names.
        assert!(!config_gen.should_generate_type(&json!({"10": {
            "name": "test",
            "age": 12
        }})));

        assert!(!config_gen.should_generate_type(&json!({"user info": {
            "age": 12
        }})));
    }

    #[test]
    fn test_new_url_query_parser() {
        let url = Url::parse(
            "http://example.com/path?query1=value1&query2=12&query3=12.3&query4=1,2,4&query5=true",
        )
        .unwrap();
        let parser = UrlQueryParser::new(&url);

        assert_eq!(parser.queries.len(), 5);

        assert_eq!(parser.queries[0].key, "query1");
        assert_eq!(parser.queries[0].data_type, "String");

        assert_eq!(parser.queries[1].key, "query2");
        assert_eq!(parser.queries[1].data_type, "Int");

        assert_eq!(parser.queries[2].key, "query3");
        assert_eq!(parser.queries[2].data_type, "Float");

        assert_eq!(parser.queries[3].key, "query4");
        assert_eq!(parser.queries[3].data_type, "List");

        assert_eq!(parser.queries[4].key, "query5");
        assert_eq!(parser.queries[4].data_type, "Boolean");
    }

    #[test]
    fn test_new_url_query_parser_empty() {
        let url = Url::parse("http://example.com/path").unwrap();
        let parser = UrlQueryParser::new(&url);
        assert_eq!(parser.queries.len(), 0);
    }
}
