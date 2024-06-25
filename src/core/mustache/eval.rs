use super::{Mustache, Segment};
use crate::core::path::{PathGraphql, PathString};

pub trait Eval {
    type In;
    type Out;

    fn eval(&self, mustache: &Mustache, in_value: &Self::In) -> Self::Out;
}

pub struct PathStringEval<A>(std::marker::PhantomData<A>);

impl<A> PathStringEval<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<A: PathString> Eval for PathStringEval<A> {
    type In = A;
    type Out = String;

    fn eval(&self, mustache: &Mustache, in_value: &Self::In) -> Self::Out {
        mustache
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.clone(),
                Segment::Expression(parts) => in_value
                    .path_string(parts)
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

pub struct PathGraphqlEval<A>(std::marker::PhantomData<A>);

impl<A> PathGraphqlEval<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<A: PathGraphql> Eval for PathGraphqlEval<A> {
    type In = A;
    type Out = String;

    fn eval(&self, mustache: &Mustache, in_value: &Self::In) -> Self::Out {
        mustache
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.to_string(),
                Segment::Expression(parts) => in_value.path_graphql(parts).unwrap_or_default(),
            })
            .collect()
    }
}

impl Mustache {
    // TODO: drop these methods and directly use the eval implementations
    pub fn render(&self, value: &impl PathString) -> String {
        PathStringEval::new().eval(self, value)
    }

    pub fn render_graphql(&self, value: &impl PathGraphql) -> String {
        PathGraphqlEval::new().eval(self, value)
    }
}
