use crate::core::config::Type;

pub trait Getter<T> { // TODO rename
    fn get(&self, key: &str) -> Option<&T>;
    fn get_mut(&mut self, key: &str) -> Option<&mut T>;
    fn insert(self, value: T) -> Self;
    fn remove(self, key: &str) -> Self
    where
        Self: Sized;
}

impl Getter<Type> for Vec<Type> {
    fn get(&self, key: &str) -> Option<&Type> {
        self.iter().find(|t| t.name == key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Type> {
        self.iter_mut().find(|t| t.name == key)
    }

    fn insert(mut self, value: Type) -> Self {
        self.push(value);
        self
    }

    fn remove(mut self, key: &str) -> Self {
        self.retain(|t| t.name != key);
        self
    }
}
