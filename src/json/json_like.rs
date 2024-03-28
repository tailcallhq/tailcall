pub trait JsonLike {
    type Output;

    // FIXME: rename to as_array
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
    fn get_key(&self, path: &str) -> Option<&Self::Output> {
        self.get_path(&[path])
    }
}
