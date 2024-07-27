use std::collections::{HashMap, HashSet};

use colored::*;

use crate::core::config::{Config, FieldName, TypeName};

pub struct Fmt {}

impl Fmt {
    pub fn heading(heading: &str) -> String {
        format!("{}", heading.bold())
    }

    pub fn meta(meta: &String) -> String {
        format!("{}", meta.yellow())
    }

    pub fn display(s: String) {
        println!("{}", s);
    }

    fn reduce<'a>(
        map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
        root: TypeName<'a>,
    ) -> Vec<Vec<(&'a str, (&'a str, &'a str))>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        #[allow(clippy::too_many_arguments)]
        fn dfs<'a>(
            map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
            ty: TypeName<'a>,
            path: Vec<(&'a str, (&'a str, &'a str))>,
            result: &mut Vec<Vec<(&'a str, (&'a str, &'a str))>>,
            visited: &mut HashSet<(TypeName<'a>, FieldName<'a>)>,
        ) {
            if let Some(fields) = map.get(&ty) {
                for (field_name, ty_of) in fields {
                    let mut new_path = path.clone();
                    new_path.push((ty.0, (field_name.0, ty_of.0)));
                    if !visited.contains(&(ty, *field_name)) {
                        visited.insert((ty, *field_name));
                        dfs(map, *ty_of, new_path, result, visited);
                        visited.remove(&(ty, *field_name));
                    }
                }
            } else {
                result.push(path);
            }
        }

        dfs(map, root, Vec::new(), &mut result, &mut visited);

        result
    }

    pub fn format_n_plus_one_queries(
        n_plus_one_info: HashMap<TypeName, HashSet<(FieldName, TypeName)>>,
        root: &str,
    ) -> String {
        let reduced = Fmt::reduce(&n_plus_one_info, TypeName(root));

        let query_paths: Vec<Vec<_>> = reduced
            .iter()
            .map(|item| {
                item.iter()
                    .map(|(_, (field_name, _))| *field_name)
                    .collect()
            })
            .collect();

        let query_data: Vec<String> = query_paths
            .iter()
            .map(|query_path| {
                let mut path = "query { ".to_string();
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

    pub fn log_n_plus_one(show_npo: bool, config: &Config) {
        let n_plus_one_info = config.n_plus_one();
        let mut message = format!("N + 1 detected: {}", n_plus_one_info.len());

        if show_npo {
            message.push('\n');
            message.push_str(&Fmt::format_n_plus_one_queries(
                n_plus_one_info,
                config.schema.query.as_deref().unwrap_or_default(),
            ));
        }

        tracing::info!("{}", message);
    }
}
