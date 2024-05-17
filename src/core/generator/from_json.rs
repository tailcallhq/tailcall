use serde_json::Value;
use url::Url;

use crate::core::config::{Arg, Config, ConfigModule, Field, Http, Type};
use crate::core::helpers::gql_type::{
    detect_gql_data_type, is_list_type, is_primitive, is_valid_field_name, to_gql_type,
};

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

    /// API used for generation of the GQL schema.
    url: Url,

    /// Used to generate the type names.
    type_counter: i32,
}

impl ConfigGenerator {
    fn new(url: &str) -> Self {
        Self {
            config: Config::default(),
            url: Url::parse(url).expect("unable to parse the url."),
            type_counter: 1,
        }
    }

    fn insert_type(&mut self, type_name: &str, actual_type: Type) {
        self.config.types.insert(type_name.to_string(), actual_type);
    }

    fn should_generate_type(&self, value: &Value) -> bool {
        match value {
            Value::Array(json_array) => !json_array.is_empty(),
            Value::Object(json_object) => {
                if json_object.is_empty() {
                    return false;
                }
                // generate type only when all fields have graphql compatible field name.
                if json_object
                    .keys()
                    .any(|json_property| !is_valid_field_name(json_property))
                {
                    return false;
                }

                true
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

    fn generate_query_type(&mut self, value: &Value, root_type_name: String) {
        let mut field = Field {
            list: is_list_type(value),
            type_of: root_type_name,
            ..Default::default()
        };

        let query_list = UrlQueryParser::new(&self.url).queries;

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
        let mut complete_path = self.url.path().to_string();
        if !path_queries.is_empty() {
            complete_path = format!("{}?{}", complete_path, path_queries.join("&"))
        }
        http.path = complete_path;
        field.http = Some(http);

        // by default query field will have root field name.
        let mut ty = Type::default();
        ty.fields.insert("root".to_string(), field);
        self.insert_type("Query", ty);
    }

    fn generate_upstream(&mut self) {
        self.config.upstream.base_url = Some(format!(
            "{}://{}",
            self.url.scheme(),
            self.url.host_str().unwrap()
        ));
    }

    fn generate_schema(&mut self) {
        self.config.schema.query = Some("Query".to_string());
    }

    fn generate(&mut self, resp: &Value) {
        let root_type_name = self.generate_types(resp);
        self.generate_query_type(resp, root_type_name);
        self.generate_upstream();
        self.generate_schema();
    }
}

pub fn from_json(url: &str, json_resp: &Value) -> ConfigModule {
    let mut ctx = ConfigGenerator::new(url);
    ctx.generate(json_resp);
    ConfigModule::from(ctx.config)
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use url::Url;

    use crate::core::generator::from_json::{ConfigGenerator, UrlQueryParser};

    #[test]
    fn test_should_generate_type() {
        let config_gen = ConfigGenerator::new("https://www.jsonplaceholder.typicode.come/users");
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
    fn test_generate_upstream() {
        let mut config_gen =
            ConfigGenerator::new("https://www.jsonplaceholder.typicode.come/users");
        config_gen.generate_upstream();
        assert!(config_gen.config.upstream.base_url.is_some())
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
