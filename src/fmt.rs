use colored::*;

use crate::config::Config;

pub struct Fmt {}

impl Fmt {
    pub fn heading(heading: &String) -> String {
        format!("{}", heading.bold())
    }

    pub fn meta(meta: &String) -> String {
        format!("{}", meta.yellow())
    }

    pub fn table(labels: Vec<(String, String)>) -> String {
        let max_length = labels.iter().map(|(key, _)| key.len()).max().unwrap_or(0) + 1;
        let padding = " ".repeat(max_length);
        let mut table = labels
            .iter()
            .map(|(key, value)| {
                Fmt::heading(
                    &(key.clone() + ":" + padding.as_str())
                        .chars()
                        .take(max_length)
                        .collect::<String>(),
                ) + " "
                    + value
            })
            .collect::<Vec<String>>()
            .join("\n");
        table.push('\n');
        table
    }

    pub fn format_n_plus_one_queries(n_plus_one_info: Vec<Vec<(String, String)>>) -> String {
        let query_paths: Vec<Vec<&String>> = n_plus_one_info
            .iter()
            .map(|item| {
                item.iter()
                    .map(|(_, field_name)| field_name)
                    .collect::<Vec<&String>>()
            })
            .collect();

        let query_data: Vec<String> = query_paths
            .iter()
            .map(|query_path| {
                let mut path = "  query { ".to_string();
                path.push_str(
                    query_path
                        .iter()
                        .rfold("".to_string(), |s, field_name| {
                            if s.is_empty() {
                                field_name.to_string()
                            } else {
                                format!("{} {{ {} }}", field_name, s)
                            }
                        })
                        .as_str(),
                );
                path.push_str(" }");
                path
            })
            .collect();

        Fmt::meta(&query_data.iter().rfold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", query, s)
            }
        }))
    }

    pub fn n_plus_one_data(n_plus_one_queries: bool, config: &Config) -> (String, String) {
        let n_plus_one_info = config.n_plus_one();
        if n_plus_one_queries {
            (
                "N + 1".to_string(),
                [
                    n_plus_one_info.len().to_string(),
                    Self::format_n_plus_one_queries(n_plus_one_info),
                ]
                .join("\n"),
            )
        } else {
            ("N + 1".to_string(), n_plus_one_info.len().to_string())
        }
    }
}
