use std::borrow::Cow;

pub trait LensPath<A>: Sized {
    fn get_path<'a>(&'a self, a: &[String]) -> Option<Cow<'a, A>>
    where
        A: Clone;
}
