use crate::core::config::Type;

pub trait Getter<T> {
    fn get(&self, key: &str) -> Option<&T>;
    fn get_mut(&mut self, key: &str) -> Option<&mut T>;
}

impl Getter<Type> for [Type] {
    fn get(&self, key: &str) -> Option<&Type> {
        self.iter().find(|t| t.name == key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Type> {
        self.iter_mut().find(|t| t.name == key)
    }
}

impl Getter<Type> for Vec<Type> {
    fn get(&self, key: &str) -> Option<&Type> {
        self.iter().find(|t| t.name == key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Type> {
        self.iter_mut().find(|t| t.name == key)
    }
}
