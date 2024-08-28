mod borrow;
mod graphql;
mod json_like;
mod json_like_list;
mod json_schema;
mod serde;

use std::collections::HashMap;

pub use json_like::*;
pub use json_like_list::*;
pub use json_schema::*;

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'json, J: JsonLike<'json>>(
    root: &'json J,
    path: &[String],
    mut vector: Vec<(&'json J, &'json J)>,
) -> Vec<(&'json J, &'json J)> {
    if let Some(root) = root.as_array() {
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

fn group_by_key<'json, J: JsonLike<'json>>(
    src: Vec<(&'json J, &'json J)>,
) -> HashMap<String, Vec<&'json J>> {
    let mut map: HashMap<String, Vec<&'json J>> = HashMap::new();
    for (key, value) in src {
        // Need to handle number and string keys
        let key_str = key
            .as_str()
            .map(|a| a.to_string())
            .or_else(|| key.as_f64().map(|a| a.to_string()));

        if let Some(key) = key_str {
            if let Some(values) = map.get_mut(&key) {
                values.push(value);
            } else {
                map.insert(key, vec![value]);
            }
        }
    }
    map
}
