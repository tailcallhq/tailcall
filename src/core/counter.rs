use std::cell::Cell;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::core::CallId;

pub trait Count: Debug {
    type Item;
    fn next(&self) -> Self::Item;
}

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Counter(Cell<usize>);
impl Counter {
    pub fn new(start: usize) -> Self {
        Self(Cell::new(start))
    }
}

impl Count for Counter {
    type Item = usize;

    fn next(&self) -> Self::Item {
        let curr = self.0.get();
        self.0.set(curr + 1);
        curr
    }
}

#[derive(Default, Debug)]
pub struct MutexCounter(Mutex<usize>);

impl Count for MutexCounter {
    type Item = CallId;

    fn next(&self) -> CallId {
        let mut x = self.0.lock().unwrap();
        *x += 1;
        CallId::new(*x)
    }
}
