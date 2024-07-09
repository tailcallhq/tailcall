use std::collections::HashMap;

use super::JsonT;
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

impl<A: JsonT> JsonLike for A {
    type Json = A;
    type JsonObject = A::JsonObject;

    fn default() -> Self {
        A::default()
    }

    fn new_array(arr: Vec<Self::Json>) -> Self {
        <A as JsonT>::new_array(arr)
    }

    fn new(value: &Self::Json) -> &Self {
        <A as JsonT>::new(value)
    }

    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str> {
        <A as JsonT>::array_ok(self).ok_or("Not an array")
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        <A as JsonT>::object_ok(self).ok_or("Not an object")
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        <A as JsonT>::str_ok(self).ok_or("Not a string")
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        <A as JsonT>::i64_ok(self).ok_or("Not an i64")
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        <A as JsonT>::u64_ok(self).ok_or("Not a u64")
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        <A as JsonT>::f64_ok(self).ok_or("Not an f64")
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        <A as JsonT>::bool_ok(self).ok_or("Not a bool")
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        <A as JsonT>::null_ok(self).ok_or("Not null")
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str> {
        <A as JsonT>::option_ok(self).ok_or("Not an option")
    }

    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&'a Self::Json> {
        <A as JsonT>::get_path(self, path)
    }
    fn get_key<'a>(&'a self, path: &'a str) -> Option<&'a Self::Json> {
        <A as JsonT>::get_key(self, path)
    }
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>> {
        <A as JsonT>::group_by(self, path)
    }
}
