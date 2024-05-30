use std::collections::HashSet;

use regex::Regex;
use url::Url;

use super::types_generator::OperationGenerator;
use crate::core::config::{Arg, Config, Field, Http, KeyValue, Type};
use crate::core::helpers::gql_type::detect_gql_data_type;

#[derive(Debug)]
struct UrlQuery {
    key: String,
    data_type: String,
    is_list: bool,
}

#[derive(Debug)]
struct UrlQueryParser {
    queries: Vec<UrlQuery>,
}

impl UrlQueryParser {
    fn new(url: &Url) -> Self {
        let mut queries: Vec<UrlQuery> = Vec::new();
        let mut seen_keys = HashSet::new();

        for (query, value) in url.query_pairs() {
            let key = query.to_string();
            let value_str = value.to_string();

            if seen_keys.contains(&key) {
                // Find the existing query and mark it as a list
                if let Some(existing_query) = queries.iter_mut().find(|q| q.key == key) {
                    existing_query.is_list = true;
                }
            } else {
                let is_list = value_str.contains(',');
                queries.push(UrlQuery {
                    key: key.clone(),
                    data_type: detect_gql_data_type(&value_str),
                    is_list,
                });
                seen_keys.insert(key);
            }
        }

        Self { queries }
    }
}

pub struct QueryGenerator<'a> {
    is_list_json: bool,
    url: &'a Url,
    query: &'a str,
    field_name: &'a str,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(is_list_json: bool, url: &'a Url, query: &'a str, field_name: &'a str) -> Self {
        Self { is_list_json, url, query, field_name }
    }
}

fn check_n_add_path_variables(field: &mut Field, http: &mut Http, url: &Url) {
    let re = Regex::new(r"/(\d+)").unwrap();
    let mut arg_index = 1;
    let path_url = url.path();

    let replaced_str = re.replace_all(path_url, |_: &regex::Captures| {
        let arg_key = format!("p{}", arg_index);
        let placeholder = format!("/{{{{.args.{}}}}}", arg_key);

        let arg = Arg {
            type_of: "Int".to_string(),
            required: true,
            ..Default::default()
        };

        field.args.insert(arg_key, arg);

        arg_index += 1;
        placeholder
    });

    // add path in http directive.
    http.path = replaced_str.to_string();
}

fn check_n_add_query_variables(field: &mut Field, http: &mut Http, url: &Url) {
    let query_list = UrlQueryParser::new(url).queries;

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
}

fn create_http_directive(field: &mut Field, url: &Url) -> Http {
    let mut http: Http = Http::default();

    check_n_add_path_variables(field, &mut http, url);
    check_n_add_query_variables(field, &mut http, url);

    http
}

impl OperationGenerator for QueryGenerator<'_> {
    fn generate(&self, root_type: &str, mut config: Config) -> Config {
        let mut field = Field {
            list: self.is_list_json,
            type_of: root_type.to_string(),
            ..Default::default()
        };

        field.http = Some(create_http_directive(&mut field, self.url));

        let mut ty = Type::default();
        ty.fields.insert(self.field_name.to_string(), field);
        config.types.insert(self.query.to_string(), ty);
        config
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::QueryGenerator;
    use crate::core::generator::json::query_generator::UrlQueryParser;
    use crate::core::generator::json::types_generator::OperationGenerator;

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

    #[test]
    fn test_list_json_query_generator() {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(true, &url, "Query", "f1");
        let config = query_generator.generate("T1", Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator() {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator.generate("T1", Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator_with_query_params() {
        let url = Url::parse("http://example.com/path?q=12&is_verified=true").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator.generate("T1", Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator_with_path_variables() {
        let url = Url::parse("http://example.com/users/12").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator.generate("T1", Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }
}
