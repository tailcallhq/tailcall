use std::cell::Cell;
use std::fmt::Debug;
use std::sync::Mutex;

use num::Num;

pub trait Count {
    type Item;
    fn next(&mut self) -> Self::Item;
}

pub struct Counter<A>(Cell<A>);
impl<A> Counter<A> {
    pub fn new(start: A) -> Self {
        Self(Cell::new(start))
    }
}

impl<A: num::Num> Count for Counter<A> {
    type Item = A;

    fn next(&mut self) -> A {
        let curr = self.0.get();
        self.0.set(curr + A::one());
        curr
    }
}

#[derive(Default, Debug)]
pub struct AtomicCounter<A>(Mutex<A>);

impl<A: num::Num> Count for AtomicCounter<A> {
    type Item = A;

    fn next(&mut self) -> A {
        let mut x = self.0.lock().unwrap();
        *x += A::one();
        *x
    }
}
