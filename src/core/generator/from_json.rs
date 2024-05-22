use serde_json::{Map, Value};
use url::Url;

use crate::core::config::{Arg, Config, Field, Http, KeyValue, Type};
use crate::core::helpers::gql_type::{
    detect_gql_data_type, is_primitive, is_valid_field_name, to_gql_type,
};
use crate::core::merge_right::MergeRight;

#[derive(Debug)]
struct UrlQuery {
    key: String,
    data_type: String, // gql type.
    is_list: bool,
}

#[derive(Debug)]
struct UrlQueryParser {
    queries: Vec<UrlQuery>,
}

impl UrlQueryParser {
    fn new(url: &Url) -> Self {
        let query_list: Vec<_> = url
            .query_pairs()
            .map(|(k, v)| UrlQuery {
                key: k.to_string(),
                data_type: detect_gql_data_type(&v),
                is_list: v.contains(","),
            })
            .collect();
        Self { queries: query_list }
    }
}

struct ConfigGenerator {
    /// final configuration that's being built up.
    config: Config,
    /// Used to generate the type names.
    type_counter: u32,
    /// Used to generate the field names.
    field_counter: u32,
}

impl ConfigGenerator {
    fn new() -> Self {
        Self { config: Config::default(), type_counter: 1, field_counter: 1 }
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

    fn create_type_from_object(&mut self, json_object: &Map<String, Value>) -> Type {
        let mut ty = Type::default();
        for (json_property, json_val) in json_object {
            let field = if !self.should_generate_type(json_val) {
                // if object, array is empty or object has in-compatible fields then
                // generate scalar for it.
                Field {
                    type_of: self.generate_scalar(),
                    list: json_val.is_array(),
                    ..Default::default()
                }
            } else {
                let mut field = Field::default();
                if is_primitive(json_val) {
                    field.type_of = to_gql_type(json_val);
                } else {
                    let type_name = self.generate_types(json_val);
                    field.type_of = type_name;
                    field.list = json_val.is_array()
                }
                field
            };
            ty.fields.insert(json_property.to_string(), field);
        }
        ty
    }

    /// given a list of types, merges all fields into single type.
    fn merge_types(type_list: Vec<Type>) -> Type {
        let mut ty = Type::default();
        for current_type in type_list {
            for (key, value) in current_type.fields {
                if let Some(existing_value) = ty.fields.get(&key) {
                    if existing_value.type_of.is_empty() || existing_value.type_of == "Empty" {
                        ty.fields.insert(key, value);
                    }
                } else {
                    ty.fields.insert(key, value);
                }
            }
        }
        ty
    }

    fn generate_types(&mut self, json_value: &Value) -> String {
        match json_value {
            Value::Array(json_arr) => {
                let vec_capacity = json_arr.first().map_or(0, |json_item| {
                    if json_item.is_object() {
                        json_arr.len()
                    } else {
                        0
                    }
                });
                let mut object_types = Vec::<_>::with_capacity(vec_capacity);
                for json_item in json_arr {
                    if let Value::Object(json_obj) = json_item {
                        if !self.should_generate_type(json_item) {
                            return self.generate_scalar();
                        }
                        object_types.push(self.create_type_from_object(json_obj));
                    } else {
                        return self.generate_types(json_item);
                    }
                }

                if !object_types.is_empty() {
                    // merge the generated types of list into single concrete type.
                    let merged_type = ConfigGenerator::merge_types(object_types);
                    let type_name = format!("T{}", self.type_counter);
                    self.type_counter += 1;
                    self.insert_type(&type_name, merged_type);
                    return type_name;
                }

                // generate a scalar if array is empty.
                self.generate_scalar()
            }
            Value::Object(json_obj) => {
                if !self.should_generate_type(json_value) {
                    return self.generate_scalar();
                }
                let ty = self.create_type_from_object(json_obj);
                let type_name = format!("T{}", self.type_counter);
                self.type_counter += 1;
                self.insert_type(&type_name, ty);
                type_name
            }
            other => to_gql_type(other),
        }
    }

    fn create_http_directive(&self, field: &mut Field, url: &Url) -> Http {
        let query_list = UrlQueryParser::new(url).queries;

        // add args to field and prepare mustache template format queries.
        let mut http: Http = Http::default();
        for query in query_list {
            let arg = Arg {
                list: query.is_list,
                type_of: query.data_type,
                required: true,
                ..Default::default()
            };

            let value: String = format!("{{{{.args.{}}}}}", query.key);
            http.query.push(KeyValue { key: query.key.clone(), value });
            field.args.insert(query.key, arg);
        }

        // add path in http directive.
        http.path = url.path().to_string();

        http
    }

    fn generate_query_type(&mut self, url: &Url, value: &Value, root_type_name: String) {
        let mut field = Field {
            list: value.is_array(),
            type_of: root_type_name,
            ..Default::default()
        };

        field.http = Some(self.create_http_directive(&mut field, url));

        let mut ty = Type::default();
        ty.fields.insert(format!("f{}", self.field_counter), field);
        self.insert_type("Query", ty);
        self.field_counter += 1;
    }

    fn generate_upstream(&mut self, url: &Url) -> anyhow::Result<()> {
        let host = url
            .host_str()
            .ok_or(anyhow::anyhow!("Failed to extract host from URL: {}", url))?;
        let base_url = match url.port() {
            Some(port) => format!("{}://{}:{}", url.scheme(), host, port),
            None => format!("{}://{}", url.scheme(), host),
        };

        self.config.upstream.base_url = Some(base_url);
        Ok(())
    }

    fn generate_schema(&mut self) {
        self.config.schema.query = Some("Query".to_string());
    }

    fn generate(&mut self, url: &str, resp: &Value) -> anyhow::Result<()> {
        let url = Url::parse(url)?;
        let root_type_name = self.generate_types(resp);
        self.generate_query_type(&url, resp, root_type_name);
        self.generate_upstream(&url)?;
        self.generate_schema();
        Ok(())
    }
}

pub struct ConfigGenerationRequest<'a> {
    url: &'a str,
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

    let unused_types = config.unused_types();
    config = config.remove_types(unused_types);

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
    fn test_generate_upstream() -> anyhow::Result<()> {
        let input_urls = [
            "http://localhost:8080/q?search=test&page=1&pageSize=20",
            "http://127.0.0.1:8000/api/v1/users",
        ];
        let expected_urls = ["http://localhost:8080", "http://127.0.0.1:8000"];

        for i in 0..input_urls.len() {
            let mut cfg_gen = ConfigGenerator::new();
            let parsed_url = Url::parse(input_urls[i]).unwrap();
            cfg_gen.generate_upstream(&parsed_url)?;
            assert_eq!(cfg_gen.config.upstream.base_url.unwrap(), expected_urls[i]);
        }
        Ok(())
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
        assert!(!parser.queries[0].is_list);

        assert_eq!(parser.queries[1].key, "query2");
        assert_eq!(parser.queries[1].data_type, "Int");
        assert!(!parser.queries[1].is_list);

        assert_eq!(parser.queries[2].key, "query3");
        assert_eq!(parser.queries[2].data_type, "Float");
        assert!(!parser.queries[2].is_list);

        assert_eq!(parser.queries[3].key, "query4");
        assert_eq!(parser.queries[3].data_type, "Int");
        assert!(parser.queries[3].is_list);

        assert_eq!(parser.queries[4].key, "query5");
        assert_eq!(parser.queries[4].data_type, "Boolean");
        assert!(!parser.queries[4].is_list);
    }

    #[test]
    fn test_new_url_query_parser_empty() {
        let url = Url::parse("http://example.com/path").unwrap();
        let parser = UrlQueryParser::new(&url);
        assert_eq!(parser.queries.len(), 0);
    }
}
