mod borrow;
mod graphql;
mod json_like;
mod json_schema;
mod serde;

use std::collections::HashMap;

pub use json_like::*;
pub use json_schema::*;

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'a, J: JsonLike<'a>>(
    root: &'a J,
    path: &'a [String],
    mut vector: Vec<(&'a J, &'a J)>,
) -> Vec<(&'a J, &'a J)>
where
    J::JsonObject: JsonObjectLike<'a>,
{
    if let Ok(root) = root.as_array_ok() {
        for value in root.iter() {
            vector = gather_path_matches(value, path, vector);
        }
    } else if let Some((key, tail)) = path.split_first() {
        if let Some(value) = root.get_key(key) {
            if tail.is_empty() {
                vector.push((value, root));
            } else {
                vector = gather_path_matches(value, tail, vector);
            }
        }
    }

    vector
}

fn group_by_key<'a, J: JsonLike<'a>>(src: Vec<(&'a J, &'a J)>) -> HashMap<String, Vec<&'a J>> {
    let mut map: HashMap<String, Vec<&'a J>> = HashMap::new();
    for (key, value) in src {
        // Need to handle number and string keys
        let key_str = key
            .as_str_ok()
            .map(|a| a.to_string())
            .or_else(|_| key.as_f64_ok().map(|a| a.to_string()));

        if let Ok(key) = key_str {
            if let Some(values) = map.get_mut(&key) {
                values.push(value);
            } else {
                map.insert(key, vec![value]);
            }
        }
    }
    map
}
