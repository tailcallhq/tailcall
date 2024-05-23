use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

static POOL: OnceLock<RwLock<IdentPool>> = OnceLock::new();

fn get_pool() -> RwLockReadGuard<'static, IdentPool> {
    POOL.get_or_init(|| RwLock::new(IdentPool::default()))
        .read()
        .unwrap()
}

fn get_pool_mut() -> RwLockWriteGuard<'static, IdentPool> {
    POOL.get_or_init(|| RwLock::new(IdentPool::default()))
        .write()
        .unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident {
    id: u64,
}

impl Ident {
    pub fn new(name: String) -> Ident {
        let mut pool_guard = get_pool_mut();
        if let Some(index) = pool_guard.get_index(&name) {
            Ident { id: index }
        } else {
            let new_index = pool_guard.len();
            pool_guard.push(name);
            Ident { id: new_index as u64 }
        }
    }

    pub fn rename(&self, new_name: String) {
        let mut pool_guard = get_pool_mut();
        pool_guard[self.id as usize] = new_name;
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pool_guard = get_pool();
        let s = pool_guard.get(self.id as usize);
        write!(f, "{}", s)
    }
}

#[derive(Default, Clone)]
pub struct IdentPool {
    pool: Vec<String>,
    index: HashMap<String, u64>,
}

impl IdentPool {
    pub fn get_index(&self, name: &str) -> Option<u64> {
        self.index.get(name).copied()
    }

    pub fn get(&self, index: usize) -> &String {
        &self.pool[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut String {
        &mut self.pool[index]
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, ident: String) {
        self.pool.push(ident)
    }

    pub fn last(&self) -> Option<&String> {
        self.pool.last()
    }
}

impl Index<usize> for IdentPool {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<usize> for IdentPool {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pool[index]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_ident_display() {
        let ident = Ident::new("Symbol".to_owned());
        assert_eq!("Symbol", ident.to_string());
    }

    #[test]
    fn test_ident_eq() {
        let ident = Ident::new("Symbol".to_owned());
        let new_ident = ident.clone();
        assert_eq!(new_ident, ident);

        assert_eq!("Symbol", ident.to_string());
    }

    #[test]
    fn test_ident_rename() {
        let ident = Ident::new("Symbol".to_owned());
        let ident2 = ident.clone();
        assert_eq!("Symbol", ident.to_string());
        assert_eq!("Symbol", ident2.to_string());

        ident.rename("NewSymbol".to_owned());
        assert_eq!("NewSymbol", ident.to_string());
        assert_eq!("NewSymbol", ident2.to_string());
    }
}
