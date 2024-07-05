use std::cell::Cell;
use std::sync::Mutex;

pub trait Count {
    type Item;
    fn next(&self) -> Self::Item;
}

#[derive(Default)]
pub struct Counter<A>(Cell<A>);
impl<A> Counter<A> {
    pub fn new(start: A) -> Self {
        Self(Cell::new(start))
    }
}

impl<A: Copy + num::Num> Count for Counter<A> {
    type Item = A;

    fn next(&self) -> A {
        let curr = self.0.get();
        self.0.set(curr + A::one());
        curr
    }
}

#[derive(Default)]
pub struct AtomicCounter<A>(Mutex<A>);

impl<A: Copy + num::Num> Count for AtomicCounter<A> {
    type Item = A;

    fn next(&self) -> A {
        let mut x = self.0.lock().unwrap();
        *x = *x + A::one();
        *x
    }
}
