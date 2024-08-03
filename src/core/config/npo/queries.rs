use std::fmt::{Display, Formatter};

use super::chunk::Chunk;
use super::FieldName;

///
/// Represents a list of query paths that can issue a N + 1 query
#[derive(Default, Debug, PartialEq)]
pub struct Queries<'a>(Vec<Vec<&'a str>>);

impl Queries<'_> {
    pub fn size(&self) -> usize {
        self.0.len()
    }
    pub fn from_chunk(chunk: Chunk<Chunk<FieldName<'_>>>) -> Queries<'_> {
        Queries(
            chunk
                .as_vec()
                .iter()
                .map(|chunk| {
                    chunk
                        .as_vec()
                        .iter()
                        .map(|field_name| field_name.as_str())
                        .collect()
                })
                .collect(),
        )
    }
}

impl<'a> Display for Queries<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let query_data: Vec<String> = self
            .0
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

        let val = query_data.iter().rfold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", query, s)
            }
        });

        f.write_str(&val)
    }
}
