use std::collections::HashSet;

use regex::Regex;
use url::Url;

use crate::core::config::{Arg, Field, Http, KeyValue};
use crate::core::helpers::gql_type::detect_gql_data_type;

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
        let re = Regex::new(r"/(\d+)").unwrap();
        let mut arg_index = 1;
        let path_url = self.url.path();

        let mustache_compatible_url = re.replace_all(path_url, |_: &regex::Captures| {
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
        self.http.path = mustache_compatible_url.to_string();
    }

    fn add_query_variables(&mut self, field: &mut Field) {
        let url_utility = UrlUtility::new(self.url);

        for query in url_utility.get_query_params() {
            let arg = Arg {
                list: query.is_list,
                type_of: query.data_type,
                required: false,
                ..Default::default()
            };

            let value: String = format!("{{{{.args.{}}}}}", query.key);
            self.http
                .query
                .push(KeyValue { key: query.key.clone(), value });
            field.args.insert(query.key, arg);
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
    use url::Url;

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
}
