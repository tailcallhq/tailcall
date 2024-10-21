use std::collections::HashSet;

use convert_case::{Case, Casing};
use regex::Regex;
use url::Url;

use crate::core::config::{Arg, Field, Http, URLQuery};
use crate::core::generator::PREFIX;
use crate::core::helpers::gql_type::detect_gql_data_type;
use crate::core::Type;

#[derive(Debug)]
struct QueryParamInfo {
    key: String,
    data_type: String,
    is_list: bool,
}

#[derive(Debug)]
struct UrlUtility<'a>(&'a Url);

impl<'a> UrlUtility<'a> {
    fn new(url: &'a Url) -> Self {
        Self(url)
    }

    pub fn get_query_params(&self) -> Vec<QueryParamInfo> {
        let mut queries: Vec<QueryParamInfo> = Vec::new();
        let mut seen_keys = HashSet::new();
        let url = self.0;
        for (query, value) in url.query_pairs() {
            let key = query.to_string();
            let value_str = value.to_string();

            if seen_keys.contains(&key) {
                // Find the existing query and mark it as a list
                if let Some(existing_query) = queries.iter_mut().find(|q| q.key == key) {
                    existing_query.is_list = true;
                }
            } else {
                queries.push(QueryParamInfo {
                    key: key.clone(),
                    data_type: detect_gql_data_type(&value_str),
                    is_list: value_str.contains(','),
                });
                seen_keys.insert(key);
            }
        }

        queries
    }
}

pub struct HttpDirectiveGenerator<'a> {
    url: &'a Url,
    http: Http,
}

impl<'a> HttpDirectiveGenerator<'a> {
    pub fn new(url: &'a Url) -> Self {
        Self { url, http: Http::default() }
    }

    fn add_path_variables(&mut self, field: &mut Field) {
        let int_regex = Regex::new(r"/\b\d+\b").unwrap();
        let uuid_regex =
            Regex::new(r"/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();

        let mut arg_index = 1;
        let path_url = self.url.path();

        let regex_map = vec![(int_regex, "Int"), (uuid_regex, "String")];

        let mustache_compatible_url =
            regex_map
                .into_iter()
                .fold(path_url.to_string(), |acc, (regex, type_of)| {
                    regex
                        .replace_all(&acc.to_string(), |_: &regex::Captures| {
                            let arg_key = format!("{}{}", PREFIX, arg_index);
                            let placeholder = format!("/{{{{.args.{}}}}}", arg_key);

                            let arg = Arg {
                                type_of: Type::from(type_of.to_owned()).into_required(),
                                ..Default::default()
                            };

                            field.args.insert(arg_key, arg);

                            arg_index += 1;
                            placeholder
                        })
                        .to_string()
                });

        // add path in http directive.
        let mut url = self.url.clone();
        url.set_path(&mustache_compatible_url);
        url.set_query(None);

        let url = url.to_string();
        let decoded = urlencoding::decode(&url).unwrap();

        self.http.url = decoded.to_string();
    }

    fn add_query_variables(&mut self, field: &mut Field) {
        let url_utility = UrlUtility::new(self.url);

        for query in url_utility.get_query_params() {
            let type_of = Type::from(query.data_type.clone());
            let type_of = if query.is_list {
                type_of.into_list()
            } else {
                type_of
            };

            let arg = Arg { type_of, ..Default::default() };

            // Convert query key to camel case for better readability.
            let query_key = query.key.to_case(Case::Camel);
            let value: String = format!("{{{{.args.{}}}}}", query_key);

            self.http
                .query
                .push(URLQuery { key: query.key.clone(), value, skip_empty: None });
            field.args.insert(query_key, arg);
        }
    }

    pub fn generate_http_directive(mut self, field: &mut Field) -> Http {
        self.add_path_variables(field);
        self.add_query_variables(field);

        self.http
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use url::Url;

    use super::HttpDirectiveGenerator;
    use crate::core::config::Field;
    use crate::core::generator::json::http_directive_generator::UrlUtility;

    #[test]
    fn test_new_url_query_parser() {
        let url = Url::parse(
            "http://example.com/path?query1=value1&query2=12&query3=12.3&query4=1,2,4&query5=true",
        )
        .unwrap();
        let url_utility = UrlUtility::new(&url);
        let query_param_list = url_utility.get_query_params();

        assert_eq!(query_param_list.len(), 5);

        assert_eq!(query_param_list[0].key, "query1");
        assert_eq!(query_param_list[0].data_type, "String");
        assert!(!query_param_list[0].is_list);

        assert_eq!(query_param_list[1].key, "query2");
        assert_eq!(query_param_list[1].data_type, "Int");
        assert!(!query_param_list[1].is_list);

        assert_eq!(query_param_list[2].key, "query3");
        assert_eq!(query_param_list[2].data_type, "Float");
        assert!(!query_param_list[2].is_list);

        assert_eq!(query_param_list[3].key, "query4");
        assert_eq!(query_param_list[3].data_type, "Int");
        assert!(query_param_list[3].is_list);

        assert_eq!(query_param_list[4].key, "query5");
        assert_eq!(query_param_list[4].data_type, "Boolean");
        assert!(!query_param_list[4].is_list);

        let url =
            Url::parse("http://example.com/path?q=1&q=2&q=3&ids=1,2,4&userids[]=1&userids[]=2")
                .unwrap();
        let url_utility = UrlUtility::new(&url);
        let query_param_list = url_utility.get_query_params();

        assert_eq!(query_param_list[0].key, "q");
        assert!(query_param_list[0].is_list);

        assert_eq!(query_param_list[1].key, "ids");
        assert!(query_param_list[1].is_list);

        assert_eq!(query_param_list[2].key, "userids[]");
        assert!(query_param_list[2].is_list);
    }

    #[test]
    fn test_new_url_query_parser_empty() {
        let url = Url::parse("http://example.com/path").unwrap();
        let parser = UrlUtility::new(&url);
        assert_eq!(parser.get_query_params().len(), 0);
    }

    #[test]
    fn test_http_directive_path_args_uuid_int() {
        let url = Url::parse("http://example.com/foo/70b0be87-d339-4395-8559-204fd368604a/bar/123")
            .unwrap();
        let http_directive = HttpDirectiveGenerator::new(&url);
        let field = &mut Field { ..Default::default() };
        let http = http_directive.generate_http_directive(field);
        let args: HashMap<String, String> = field
            .args
            .iter()
            .map(|(name, arg)| (name.to_string(), arg.type_of.name().to_owned()))
            .collect::<HashMap<_, _>>();
        let test_args = vec![
            ("GEN__1".to_string(), "Int".to_string()),
            ("GEN__2".to_string(), "String".to_string()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert_eq!(
            "http://example.com/foo/{{.args.GEN__2}}/bar/{{.args.GEN__1}}",
            http.url
        );
        assert_eq!(test_args, args);
    }
}
