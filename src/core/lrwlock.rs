#[deny(clippy::mut_from_ref)]

use std::cell::UnsafeCell;

use thread_id;

/// A RwLock that leverages thread locals to avoid contention.
pub struct LrwLock<A: Send> {
    inner: Vec<UnsafeCell<A>>,
}

unsafe impl<A: Send> Sync for LrwLock<A> {}

// TODO: Should be configured based on the worker count
const SIZE: usize = 16;

impl<A: Clone + Send> LrwLock<A> {
    /// Create a new LrwLock.
    pub fn new(value: A) -> Self {
        let mut inner = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            inner.push(UnsafeCell::new(value.clone()));
        }

        Self { inner }
    }

    /// Lock the LrwLock for reading.
    pub fn get(&self) -> &A {
        let id = thread_id::get();

        unsafe { get_mut(&self.inner[id % SIZE]) }
    }

    pub fn modify(&self, f: impl FnOnce(&mut A)) {
        let id = thread_id::get();

        unsafe {
            f(get_mut(&self.inner[id % SIZE]));
        }
    }
}


unsafe fn get_mut<T>(ptr: &UnsafeCell<T>) -> &mut T {
    unsafe { &mut *ptr.get() }
}
