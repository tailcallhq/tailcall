use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thread_local::ThreadLocal;

/// A RwLock that leverages thread locals to avoid contention.
pub struct LrwLock<A: Send> {
    inner: ThreadLocal<RwLock<A>>,
    value: A,
}

impl<A: Clone + Send> LrwLock<A> {
    /// Create a new LrwLock.
    pub fn new(value: A) -> Self {
        Self { inner: ThreadLocal::new(), value }
    }

    /// Lock the LrwLock for reading.
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, A>> {
        self.inner.get_or(|| RwLock::new(self.value.clone())).read()
    }

    /// Lock the LrwLock for writing.
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, A>> {
        self.inner
            .get_or(|| RwLock::new(self.value.clone()))
            .write()
    }
}