use std::collections::HashSet;

use regex::Regex;
use url::Url;

use super::types_generator::OperationGenerator;
use crate::core::config::{Arg, Config, Field, Http, KeyValue, Type};
use crate::core::helpers::gql_type::detect_gql_data_type;
use crate::core::valid::Valid;

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

pub struct QueryGenerator<'a> {
    is_json_list: bool,
    url: &'a Url,
    query: &'a str,
    field_name: &'a str,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(is_json_list: bool, url: &'a Url, query: &'a str, field_name: &'a str) -> Self {
        Self { is_json_list, url, query, field_name }
    }

    fn add_path_variables(&self, field: &mut Field, http: &mut Http, url: &Url) {
        let re = Regex::new(r"/(\d+)").unwrap();
        let mut arg_index = 1;
        let path_url = url.path();

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
        http.path = mustache_compatible_url.to_string();
    }

    fn add_query_variables(&self, field: &mut Field, http: &mut Http, url: &Url) {
        let url_utility = UrlUtility::new(url);

        for query in url_utility.get_query_params() {
            let arg = Arg {
                list: query.is_list,
                type_of: query.data_type,
                required: true, /* TODO: currently non-null args are not supported, fix this
                                 * later on. */
                ..Default::default()
            };

            let value: String = format!("{{{{.args.{}}}}}", query.key);
            http.query.push(KeyValue { key: query.key.clone(), value });
            field.args.insert(query.key, arg);
        }
    }

    fn create_http_directive(&self, field: &mut Field, url: &Url) -> Http {
        let mut http: Http = Http::default();

        self.add_path_variables(field, &mut http, url);
        self.add_query_variables(field, &mut http, url);

        http
    }
}

impl OperationGenerator for QueryGenerator<'_> {
    fn generate(&self, root_type: &str, mut config: Config) -> Valid<Config, String> {
        let mut field = Field {
            list: self.is_json_list,
            type_of: root_type.to_owned(),
            ..Default::default()
        };

        field.http = Some(self.create_http_directive(&mut field, self.url));

        if let Some(type_) = config.types.get_mut(self.query) {
            type_.fields.insert(self.field_name.to_owned(), field);
        } else {
            let mut ty = Type::default();
            ty.fields.insert(self.field_name.to_owned(), field);
            config.types.insert(self.query.to_owned(), ty);
        }
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::QueryGenerator;
    use crate::core::generator::json::query_generator::UrlUtility;
    use crate::core::generator::json::types_generator::OperationGenerator;
    use crate::core::valid::Validator;

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
    fn test_list_json_query_generator() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(true, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator_with_query_params() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/path?q=12&is_verified=true").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }

    #[test]
    fn test_query_generator_with_path_variables() -> anyhow::Result<()> {
        let url = Url::parse("http://example.com/users/12").unwrap();
        let query_generator = QueryGenerator::new(false, &url, "Query", "f1");
        let config = query_generator
            .generate("T1", Default::default())
            .to_result()?;
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
