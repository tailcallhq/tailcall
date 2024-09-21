#![allow(dead_code)]
use std::ops::Deref;

use hyper::header::{HeaderName, HeaderValue};
use serde::Serialize;

///
/// Just an empty wrapper around a value used to implement foreign traits for
/// foreign types.
#[derive(Clone, Debug, PartialEq, Eq, std::hash::Hash)]
pub struct Lift<A>(A);

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

impl Serialize for Lift<reqwest::Method> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

// Serialize impl for HeaderValue.
impl Serialize for Lift<HeaderValue> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.to_str().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(s)
    }
}

impl Serialize for Lift<HeaderName> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}
