use std::ops::Deref;

///
/// Just an empty wrapper around a value used to implement `From` for foreign
/// types.
pub struct Lift<A>(A);
impl<A> Deref for Lift<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> Lift<A> {
    #[allow(dead_code)]
    pub fn take(self) -> A {
        self.0
    }
}

impl<A> From<A> for Lift<A> {
    fn from(a: A) -> Self {
        Lift(a)
    }
}
