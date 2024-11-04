use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thread_id;

/// A RwLock that leverages thread locals to avoid contention.
pub struct LrwLock<A: Send> {
    inner: Vec<RwLock<A>>,
}

const SIZE: usize = 16;

impl<A: Clone + Send> LrwLock<A> {
    /// Create a new LrwLock.
    pub fn new(value: A) -> Self {
        let mut inner = Vec::with_capacity(SIZE);
        for _ in 1..SIZE {
            inner.push(RwLock::new(value.clone()));
        }

        Self { inner }
    }

    /// Lock the LrwLock for reading.
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, A>> {
        let id = thread_id::get();

        self.inner[id % SIZE].read()
    }

    /// Lock the LrwLock for writing.
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, A>> {
        let id = thread_id::get();

        self.inner[id % SIZE].write()
    }
}
