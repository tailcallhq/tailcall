use std::cell::Cell;

#[allow(unused)]
#[derive(Default)]
pub struct Counter(Cell<usize>);
impl Counter {
    pub fn new(start: usize) -> Self {
        Self(Cell::new(start))
    }
    pub fn next(&self) -> usize {
        let curr = self.0.get();
        self.0.set(curr + 1);
        curr
    }
}
