use std::cell::RefCell;

#[allow(unused)]
#[derive(Default)]
pub struct Counter(RefCell<usize>);
impl Counter {
    pub fn next(&self) -> usize {
        self.0.replace_with(|a| *a + 1)
    }
}
