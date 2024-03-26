use std::collections::HashMap;

pub trait JsonLike {
    type Output;
    fn as_array_ok(&self) -> Option<&Vec<Self::Output>>;
    fn as_str_ok(&self) -> Option<&str>;
    fn as_string_ok(&self) -> Option<&String>;
    fn as_i64_ok(&self) -> Option<i64>;
    fn as_u64_ok(&self) -> Option<u64>;
    fn as_f64_ok(&self) -> Option<f64>;
    fn as_bool_ok(&self) -> Option<bool>;
    fn as_null_ok(&self) -> Option<()>;

    // FIXME: rename to get_path_value
    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output>;

    // Convertors
    fn from_output(value: &Self::Output) -> &Self;
    fn to_output(value: &Self) -> &Self::Output;

    // Default implementations
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>>
    where
        Self: Sized,
        Self::Output: JsonLike,
    {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn get_key(&self, path: &str) -> Option<&Self::Output> {
        self.get_path(&[path])
    }
}

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'a, J: JsonLike>(
    root: &'a J,
    path: &'a [String],
    mut vector: Vec<(&'a J::Output, &'a J::Output)>,
) -> Vec<(&'a J::Output, &'a J::Output)> {
    if let Some(root) = root.as_array_ok() {
        for value in root {
            vector = gather_path_matches(J::from_output(value), path, vector);
        }
    } else if let Some((key, tail)) = path.split_first() {
        if let Some(value) = root.get_key(key) {
            if tail.is_empty() {
                vector.push((value, J::to_output(root)));
            } else {
                vector = gather_path_matches(J::from_output(value), tail, vector);
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
