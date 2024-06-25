use crate::core::path::{PathGraphql, PathString};

use super::{Mustache, Segment};

pub trait Eval {
    type In;
    type Out;

    fn eval(mustache: Mustache, in_value: Self::In) -> Self::Out;
}

#[derive(Default)]
pub struct PathStringEval<A>(std::marker::PhantomData<A>);

impl<A: PathString> Eval for PathStringEval<A> {
    type In = A;
    type Out = String;

    fn eval(mustache: Mustache, in_value: Self::In) -> Self::Out {
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

impl<A: PathGraphql> Eval for PathGraphqlEval<A> {
    type In = A;
    type Out = String;

    fn eval(mustache: Mustache, in_value: Self::In) -> Self::Out {
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
