use crate::blueprint::Blueprint;
use crate::digest::Digest;
use std::collections::HashMap;
use std::sync::Arc;

// TODO: This is in-memory or redis
pub struct SchemaRegistry {
    data: HashMap<Digest, Arc<Blueprint>>,
}

// For now I'll just hold a lock on the entire thing.
// This is not thread safe. Caller must ensure proper mutexes are enforced.
impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry {
            data: HashMap::new(),
        }
    }

    /// If blueprint present => updated and old blueprint returned
    /// If blueprint not present => added and none returned
    pub fn add(&mut self, digest: Digest, blueprint: Blueprint) -> Option<Arc<Blueprint>> {
        self.data.insert(digest, Arc::new(blueprint))
    }

    pub fn get(&self, digest: &Digest) -> Option<Arc<Blueprint>> {
        self.data.get(digest).map(|bp| bp.clone())
    }

    pub fn drop(&mut self, digest: &Digest) -> Result<(), RegistryError> {
        todo!()
    }

    pub fn list(&self, start: usize, end: usize) -> Vec<Arc<Blueprint>> {
        todo!()
    }
}

// TODO: Better error type
type RegistryError = &'static str;
