use std::collections::{HashMap, HashSet};

use colored::*;

use crate::core::config::{Config, FieldName, TypeName};

pub struct Fmt {}

impl Fmt {
    pub fn heading(heading: &str) -> String {
        format!("{}", heading.bold())
    }

    pub fn meta(meta: String) -> String {
        format!("{}", meta.yellow())
    }

    pub fn display(s: String) {
        println!("{}", s);
    }

    fn foo<'a>(
        map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
        ty: &TypeName<'a>,
        visited: &mut HashSet<(TypeName<'a>, FieldName<'a>)>,
    ) -> String {
        if let Some(val) = map.get(ty) {
            val.iter().map(|(field_name, ty_of)| {
                let nested = Fmt::foo(map, ty_of, visited);
                let val = if nested.is_empty() || visited.contains(&(*ty, *field_name)) {
                    field_name.to_string()
                } else {
                    format!("{} {{ {} }}", field_name, nested)
                };
                visited.insert((*ty, *field_name));
                val
            }).collect::<Vec<_>>().join("\n")
        } else {
            String::new()
        }
    }

    pub fn format_n_plus_one_queries(n_plus_one_info: HashMap<TypeName, HashSet<(FieldName, TypeName)>>, root: &str) -> String {
        let ans = format!("query {{ {} }}", Self::foo(&n_plus_one_info, &TypeName(root), &mut HashSet::new()));
        Fmt::meta(ans)
        /*        // println!("{}", serde_json::to_string_pretty(&n_plus_one_info).unwrap());
                let query_paths = n_plus_one_info
                    .values()
                    .map(|val| val.iter().copied().collect::<Vec<_>>())
                    .collect::<Vec<_>>();

                let query_data: Vec<String> = query_paths
                    .iter()
                    .map(|query_path| {
                        let mut path = "query { ".to_string();
                        path.push_str(
                            query_path
                                .iter()
                                .rfold("".to_string(), |s, (field_name, ty_of)| {
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
                }))*/
    }

    pub fn log_n_plus_one(show_npo: bool, config: &Config) {
        let n_plus_one_info = config.n_plus_one();
        let mut message = format!("N + 1 detected: {}", n_plus_one_info.len());

        if show_npo {
            message.push('\n');
            message.push_str(&Fmt::format_n_plus_one_queries(n_plus_one_info, config.schema.query.as_ref().map(|v| v.as_str()).unwrap_or_default()));
        }

        tracing::info!("{}", message);
    }
}
