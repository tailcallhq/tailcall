use std::collections::HashMap;

use crate::core::json::json_object_like::JsonObjectLike;

pub trait JsonLike {
    type Json;
    type JsonObject: JsonObjectLike;

    // Constructors
    fn default() -> Self;
    fn new_array(arr: Vec<Self::Json>) -> Self;
    fn new(value: &Self::Json) -> &Self;

    // Operators
    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str>;
    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str>;
    fn as_str_ok(&self) -> Result<&str, &str>;
    fn as_i64_ok(&self) -> Result<i64, &str>;
    fn as_u64_ok(&self) -> Result<u64, &str>;
    fn as_f64_ok(&self) -> Result<f64, &str>;
    fn as_bool_ok(&self) -> Result<bool, &str>;
    fn as_null_ok(&self) -> Result<(), &str>;
    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str>;
    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&'a Self::Json>;
    fn get_key<'a>(&'a self, path: &'a str) -> Option<&'a Self::Json>;
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>>;
}
