use std::borrow::Cow;

pub trait LensPath<A> {
    fn get_path<'a>(&'a self, a: &[String]) -> Option<Cow<'a, A>>
    where
        A: Clone;
}

pub trait LensPathOperation<A, B> {
    fn map(&self, f: impl FnOnce(Cow<A>) -> Cow<B>) -> LensPathMap<A, B>
    where
        A: Clone,
        B: Clone;
}

impl<A: Clone, B: Clone, X: LensPath<A> + Clone> LensPathOperation<A, B> for X {
    fn map(&self, f: impl FnOnce(Cow<A>) -> Cow<B>) -> LensPathMap<A, B> {
        LensPathMap { lens: Box::new(self.clone()), f: Box::new(f) }
    }
}

pub struct LensPathMap<A: Clone, B: Clone> {
    lens: Box<dyn LensPath<A>>,
    f: Box<dyn FnOnce(Cow<A>) -> Cow<B>>,
}

impl<A: Clone, B: Clone> LensPath<B> for LensPathMap<A, B> {
    fn get_path<'a>(&'a self, a: &[String]) -> Option<Cow<'a, B>>
    where
        B: Clone,
    {
        self.lens.get_path(a).map(|b| (self.f)(b))
    }
}
