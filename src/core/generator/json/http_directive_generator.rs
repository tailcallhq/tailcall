use std::collections::{HashMap, HashSet};

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
    route: Option<&'a String>,
    http: Http,
}

impl<'a> HttpDirectiveGenerator<'a> {
    pub fn new(url: &'a Url, route: Option<&'a String>) -> Self {
        Self { url, route, http: Http::default() }
    }

    fn add_path_variables(&mut self, field: &mut Field) {
        let mut mustache_compatible_url = String::new();

        if let Some(route) = self.route {
            let mut segments_vars: HashMap<usize, &str> = HashMap::new();
            let route_segments = route.split('/').filter(|s| !s.is_empty());

            for (segment_pos, segment) in route_segments.enumerate() {
                if let Some(variable) = segment.strip_prefix("$") {
                    segments_vars.insert(segment_pos, variable);
                }
            }

            if let Some(segments) = self.url.path_segments() {
                let mut arg_index = 1;
                for (segment_pos, segment) in segments.enumerate() {
                    if !segments_vars.contains_key(&segment_pos) {
                        mustache_compatible_url.push('/');
                        mustache_compatible_url.push_str(segment);
                        continue;
                    }

                    let arg_key = format!("p{}", arg_index);
                    let placeholder = format!("/{{{{.args.{}}}}}", arg_key);
                    mustache_compatible_url.push_str(placeholder.as_str());
                    arg_index += 1;
                    field.args.insert(
                        arg_key,
                        Arg {
                            type_of: Self::determine_arg_type_from_route_segment(segment),
                            required: true,
                            ..Default::default()
                        },
                    );
                }
            }
        } else {
            // For best-effort detection, we're going to assume these properties:
            // REST route usually starts with a static segment followed by an
            // identifier segment, e.g., albums/wpa.
            // We can determine a valid route by counting the total number of segments.
            if let Some(url_segment) = self.url.path_segments() {
                let mut peekable_url_segments = url_segment.peekable();
                // Case 1: v1/albums/wpa
                // Case 2: v1/api/albums/wpa
                // Case 3: api/albums/wpa
                let api_version_regex = Regex::new(r"v[0-9]+").unwrap();
                while let Some(first_segment) = peekable_url_segments.peek() {
                    if api_version_regex.is_match(first_segment) || first_segment.starts_with("api")
                    {
                        mustache_compatible_url.push('/');
                        mustache_compatible_url.push_str(first_segment);
                        peekable_url_segments.next();
                    } else {
                        break;
                    }
                }

                let mut arg_index = 1;
                for (segment_pos, segment) in peekable_url_segments.enumerate() {
                    if segment_pos % 2 == 1 {
                        let arg_key = format!("p{}", arg_index);
                        let placeholder = format!("/{{{{.args.{}}}}}", arg_key);
                        mustache_compatible_url.push_str(placeholder.as_str());
                        arg_index += 1;
                        field.args.insert(
                            arg_key,
                            Arg {
                                type_of: Self::determine_arg_type_from_route_segment(segment),
                                required: true,
                                ..Default::default()
                            },
                        );
                    } else {
                        mustache_compatible_url.push('/');
                        mustache_compatible_url.push_str(segment);
                    }
                }
            }
        }

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

    fn determine_arg_type_from_route_segment(segment: &str) -> String {
        let is_digits = segment.chars().all(|item| item.is_ascii_digit());
        if is_digits {
            return String::from("Int");
        }

        String::from("String")
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use crate::core::config::{Arg, Field};
    use crate::core::generator::json::http_directive_generator::{
        HttpDirectiveGenerator, UrlUtility,
    };

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

    fn add_arg_to_field(field: &mut Field, key: &str, type_of: &str) {
        field.args.insert(
            key.to_string(),
            Arg {
                type_of: type_of.to_string(),
                required: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_variable_detection_with_route_provided() {
        let url = Url::parse("http://example.com/v1/albums/wpa/photos/2").unwrap();
        let route = Some("/v1/albums/$album_name/photos/$id".to_string());

        let mut http_directive_gen = HttpDirectiveGenerator::new(&url, route.as_ref());
        let mut actual: Field = Default::default();
        http_directive_gen.add_path_variables(&mut actual);

        let mut expected: Field = Default::default();
        add_arg_to_field(&mut expected, "p1", "String");
        add_arg_to_field(&mut expected, "p2", "Int");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_variable_detection_with_best_effort() {
        let url = Url::parse("http://example.com/v22/api/albums/wpa22/photos/2/delete").unwrap();

        let mut http_directive_gen = HttpDirectiveGenerator::new(&url, None);
        let mut actual: Field = Default::default();
        http_directive_gen.add_path_variables(&mut actual);

        let mut expected: Field = Default::default();
        add_arg_to_field(&mut expected, "p1", "String");
        add_arg_to_field(&mut expected, "p2", "Int");

        assert_eq!(actual, expected);
    }
}
