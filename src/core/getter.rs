use crate::core::config::{Enum, Type};

pub trait GetName {
    fn get_name(&self) -> &str;
}

impl GetName for Type {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl GetName for Enum {
    fn get_name(&self) -> &str {
        &self.name
    }
}

pub trait Getter<T> {
    // TODO rename
    fn get(&self, key: &str) -> Option<&T>;
    fn get_mut(&mut self, key: &str) -> Option<&mut T>;
    fn insert(self, value: T) -> Self;
    fn remove(self, key: &str) -> Self
    where
        Self: Sized;
}

impl<T: GetName> Getter<T> for Vec<T> {
    fn get(&self, key: &str) -> Option<&T> {
        self.iter().find(|t| t.get_name() == key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.iter_mut().find(|t| t.get_name() == key)
    }

    fn insert(mut self, value: T) -> Self {
        self.push(value);
        self
    }

    fn remove(mut self, key: &str) -> Self {
        self.retain(|t| t.get_name() != key);
        self
    }
}
