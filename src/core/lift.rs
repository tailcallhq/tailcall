#![allow(dead_code)]
use std::ops::Deref;
use std::str::FromStr;

use hyper::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

///
/// Just an empty wrapper around a value used to implement foreign traits for
/// foreign types.
#[derive(Clone, PartialEq, Eq, std::hash::Hash)]
pub struct Lift<A>(A);

impl<A: std::fmt::Debug> std::fmt::Debug for Lift<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<A: Clone> Lift<A> {
    pub fn clone_inner(&self) -> A {
        self.0.clone()
    }
}

impl<A> Deref for Lift<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> AsRef<A> for Lift<A> {
    fn as_ref(&self) -> &A {
        &self.0
    }
}

impl<A> Lift<A> {
    pub fn take(self) -> A {
        self.0
    }
}

impl<A> From<A> for Lift<A> {
    fn from(a: A) -> Self {
        Lift(a)
    }
}

pub trait CanLift: Sized {
    fn lift(self) -> Lift<Self>;
}

impl<A> CanLift for A {
    fn lift(self) -> Lift<Self> {
        Lift::from(self)
    }
}

pub trait AsStr {
    fn as_str_value(&self) -> anyhow::Result<&str>;
}

impl AsStr for HeaderName {
    fn as_str_value(&self) -> anyhow::Result<&str> {
        Ok(self.as_str())
    }
}

impl AsStr for reqwest::Method {
    fn as_str_value(&self) -> anyhow::Result<&str> {
        Ok(self.as_str())
    }
}

impl AsStr for HeaderValue {
    fn as_str_value(&self) -> anyhow::Result<&str> {
        Ok(self.to_str()?)
    }
}

impl<T: AsStr> Serialize for Lift<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.as_str_value().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(s)
    }
}

impl<'de, T> Deserialize<'de> for Lift<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s)
            .map(Lift)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use hyper::header::{HeaderName, HeaderValue};
    use reqwest::Method;
    use serde_json;

    use super::*;

    #[test]
    fn test_request_method() {
        let method = Lift(Method::POST);
        let serialized = serde_json::to_string(&method).unwrap();
        assert_eq!(serialized, "\"POST\"");

        let deserialized: Lift<Method> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, method);
    }

    #[test]
    fn test_header_name() {
        let header_name = Lift(HeaderName::from_static("content-type"));
        let serialized = serde_json::to_string(&header_name).unwrap();
        assert_eq!(serialized, "\"content-type\"");

        let deserialized: Lift<HeaderName> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, header_name);
    }

    #[test]
    fn test_header_value() {
        let header_value = Lift(HeaderValue::from_static("application/json"));
        let serialized = serde_json::to_string(&header_value).unwrap();
        assert_eq!(serialized, "\"application/json\"");

        let deserialized: Lift<HeaderValue> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, header_value);
    }
}
