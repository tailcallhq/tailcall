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

    /*    fn get_cloned(&self, key: &str) -> Option<Type> {
        self.iter().find(|t| t.name == key).cloned()
    }*/
}

impl Getter<Type> for Vec<Type> {
    fn get(&self, key: &str) -> Option<&Type> {
        self.iter().find(|t| t.name == key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Type> {
        self.iter_mut().find(|t| t.name == key)
    }

    /*    fn get_cloned(&self, key: &str) -> Option<Type> {
        self.iter().find(|t| t.name == key).cloned()
    }*/
}

/*pub trait Getter<T, I>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
{
    fn get(iter: I, key: &str) -> Option<&T>;
    fn get_mut(iter: I, key: &str) -> Option<&mut T>;
    fn get_cloned(iter: I, key: &str) -> Option<T>;
}

impl<I: AsRef<[Type]>> Getter<Type, I> for Vec<Type> {
    fn get(iter: I, key: &str) -> Option<&Type> {
        iter.as_ref().iter().find(|t| t.name == key)
    }

    fn get_mut(iter: I, key: &str) -> Option<&mut Type> {
        iter.as_ref().iter_mut().find(|t| t.name == key)
    }

    fn get_cloned(iter: I, key: &str) -> Option<Type> {
        iter.as_ref().iter().find(|t| t.name == key).cloned()
    }
}*/
