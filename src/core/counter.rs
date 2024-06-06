use std::cell::Cell;

#[allow(unused)]
pub struct Counter(Cell<usize>);
impl Counter {
    #[allow(unused)]
    pub fn next(&self) -> usize {
        let curr = self.0.get();
        self.0.set(curr + 1);
        curr
    }
}

impl Default for Counter {
    fn default() -> Self {
        Counter(Cell::new(1))
    }

}