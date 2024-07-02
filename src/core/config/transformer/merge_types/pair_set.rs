use std::collections::HashSet;
use std::hash::Hash;

///
/// A special hashset that can store two values of the same type as a pair.
#[derive(Default)]
pub struct PairSet<A> {
    visited: HashSet<(A, A)>,
}

impl<A: PartialEq + Hash + Eq + Clone> PairSet<A> {
    pub fn insert(&mut self, a1: A, a2: A) {
        self.visited.insert((a1, a2));
    }

    pub fn contains(&self, a1: &A, a2: &A) -> bool {
        self.visited.contains(&(a1.to_owned(), a2.to_owned()))
            || self.visited.contains(&(a2.to_owned(), a1.to_owned()))
    }
}
