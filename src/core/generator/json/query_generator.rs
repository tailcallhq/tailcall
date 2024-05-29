use regex::Regex;
use url::Url;

use super::ConfigGenerator;
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
        let query_list: Vec<_> = url
            .query_pairs()
            .map(|(k, v)| UrlQuery {
                key: k.to_string(),
                data_type: detect_gql_data_type(&v),
                is_list: v.contains(","), // TODO: improve this.
            })
            .collect();
        Self { queries: query_list }
    }
}

pub struct QueryGenerator<'a> {
    is_list_json: bool,
    root_type_name: &'a str,
    field_name: &'a str,
    query_name: &'a str,
    url: &'a Url,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        is_list_json: bool,
        root_type_name: &'a str,
        field_name: &'a str,
        query_name: &'a str,
        url: &'a Url,
    ) -> Self {
        Self { is_list_json, root_type_name, url, field_name, query_name }
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

    let base_url = match url.host_str() {
        Some(host) => match url.port() {
            Some(port) => format!("{}://{}:{}", url.scheme(), host, port),
            None => format!("{}://{}", url.scheme(), host),
        },
        None => return http,
    };

    http.base_url = Some(base_url);

    http
}

impl ConfigGenerator for QueryGenerator<'_> {
    fn apply(&mut self, mut config: Config) -> Config {
        let mut field = Field {
            list: self.is_list_json,
            type_of: self.root_type_name.to_string(),
            ..Default::default()
        };

        field.http = Some(create_http_directive(&mut field, self.url));

        let mut ty = Type::default();
        ty.fields.insert(self.field_name.to_string(), field);
        config.types.insert(self.query_name.to_string(), ty);
        config
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::QueryGenerator;
    use crate::core::generator::json::query_generator::UrlQueryParser;
    use crate::core::generator::json::ConfigGenerator;

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
        let mut query_generator = QueryGenerator::new(true, "T1", "f1", "Query", &url);
        let config = query_generator.apply(Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator() {
        let url = Url::parse("http://example.com/path").unwrap();
        let mut query_generator = QueryGenerator::new(false, "T1", "f1", "Query", &url);
        let config = query_generator.apply(Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator_with_query_params() {
        let url = Url::parse("http://example.com/path?q=12&is_verified=true").unwrap();
        let mut query_generator = QueryGenerator::new(false, "T1", "f1", "Query", &url);
        let config = query_generator.apply(Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_query_generator_with_path_variables() {
        let url = Url::parse("http://example.com/users/12").unwrap();
        let mut query_generator = QueryGenerator::new(false, "T1", "f1", "Query", &url);
        let config = query_generator.apply(Default::default());
        insta::assert_snapshot!(config.to_sdl());
    }
}
