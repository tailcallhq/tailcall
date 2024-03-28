use std::collections::HashMap;

use super::*;

pub trait JsonLikeGroupBy {
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>>
    where
        Self: JsonLike,
        Self: Sized,
        Self::Output: JsonLike;
}

impl<A> JsonLikeGroupBy for A
where
    A: JsonLike<Output = A>,
{
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a A::Output>>
    where
        Self: Sized,
        A::Output: JsonLike,
    {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'a, J: JsonLike<Output = J>>(
    root: &'a J,
    path: &'a [String],
    mut vector: Vec<(&'a J::Output, &'a J::Output)>,
) -> Vec<(&'a J::Output, &'a J::Output)> {
    if let Some(root) = root.as_array_ok() {
        for value in root {
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

pub(crate) fn group_by_key<'a, J: JsonLike>(
    src: Vec<(&'a J, &'a J)>,
) -> HashMap<String, Vec<&'a J>> {
    let mut map: HashMap<String, Vec<&'a J>> = HashMap::new();
    for (key, value) in src {
        // Need to handle number and string keys
        let key_str = key
            .as_string_ok()
            .cloned()
            .or_else(|| key.as_f64_ok().map(|a| a.to_string()));

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
