use std::fmt::Debug;
use std::marker::PhantomData;

use crate::valid::{Valid, ValidationError};

pub trait Bool {}

pub struct True;
impl Bool for True {}

pub struct False;
impl Bool for False {}

pub trait Nat: Debug {}

#[derive(Debug)]
pub struct Zero;
impl Nat for Zero {}

#[derive(Debug)]
pub struct Suc<N: Nat>(PhantomData<N>);
impl<N: Nat> Nat for Suc<N> {}

pub type One = Suc<Zero>;
pub type Two = Suc<One>;
pub type Three = Suc<Two>;
pub type Four = Suc<Three>;
pub type Five = Suc<Four>;

pub trait IsEqual<N: Nat> {
    type Result: Bool;
}

impl IsEqual<Zero> for Zero {
    type Result = True;
}

impl<N: Nat> IsEqual<Suc<N>> for Zero {
    type Result = False;
}

impl<N: Nat> IsEqual<Zero> for Suc<N> {
    type Result = False;
}

impl<Lhs: Nat + IsEqual<Rhs>, Rhs: Nat> IsEqual<Suc<Rhs>> for Suc<Lhs> {
    type Result = <Lhs as IsEqual<Rhs>>::Result;
}

pub trait IsGreaterThan<N: Nat> {
    type Result: Bool;
}

impl IsGreaterThan<Zero> for Zero {
    type Result = False;
}

impl<N: Nat> IsGreaterThan<Suc<N>> for Zero {
    type Result = False;
}

impl<N: Nat> IsGreaterThan<Zero> for Suc<N> {
    type Result = True;
}

impl<Lhs: Nat + IsGreaterThan<Rhs>, Rhs: Nat> IsGreaterThan<Suc<Rhs>> for Suc<Lhs> {
    type Result = <Lhs as IsGreaterThan<Rhs>>::Result;
}

pub trait IsGreaterThanEqualTo<N: Nat> {
    type Result: Bool;
}

impl IsGreaterThanEqualTo<Zero> for Zero {
    type Result = True;
}

impl<N: Nat> IsGreaterThanEqualTo<Suc<N>> for Zero {
    type Result = False;
}

impl<N: Nat> IsGreaterThanEqualTo<Zero> for Suc<N> {
    type Result = True;
}

impl<Lhs: Nat + IsGreaterThanEqualTo<Rhs>, Rhs: Nat> IsGreaterThanEqualTo<Suc<Rhs>> for Suc<Lhs> {
    type Result = <Lhs as IsGreaterThanEqualTo<Rhs>>::Result;
}

pub trait Tupleness<N: Nat = Zero> {}

impl<T> Tupleness<Zero> for T {}

impl<A, B, An: Nat> Tupleness<Suc<An>> for (A, B) where A: Tupleness<An> {}

pub trait Tuple: Debug {}

#[derive(Debug)]
pub struct EmptyTuple;
impl Tuple for EmptyTuple {}

#[derive(Debug)]
pub struct Cons<V: Debug, T: Tuple, N: Nat, M: Nat>(V, T, PhantomData<(N, M)>);
impl<V: Debug, T: Tuple, N: Nat, M: Nat> Tuple for Cons<V, T, N, M> {}

pub trait TupleEqual<Rhs: Tuple> {
    type Result: Bool;
}

impl TupleEqual<EmptyTuple> for EmptyTuple {
    type Result = True;
}

impl<V: Debug, T: Tuple, N: Nat, M: Nat> TupleEqual<Cons<V, T, N, M>> for EmptyTuple {
    type Result = False;
}

impl<V: Debug, T: Tuple, N: Nat, M: Nat> TupleEqual<EmptyTuple> for Cons<V, T, N, M> {
    type Result = False;
}

impl<
        VR: Debug,
        TR: Tuple,
        NR: Nat,
        MR: Nat,
        VL: Debug,
        TL: Tuple + TupleEqual<TR>,
        NL: Nat,
        ML: Nat,
    > TupleEqual<Cons<VR, TR, NR, MR>> for Cons<VL, TL, NL, ML>
{
    type Result = <TL as TupleEqual<TR>>::Result;
}

pub trait Append<Val: Debug>
where
    Self: Tuple,
{
    type Result: Tuple;

    fn append(self, val: Val) -> Self::Result;
}

impl<V: Debug, Val: Debug, N: Nat + IsEqual<One, Result = True>, M: Nat> Append<Val>
    for Cons<V, EmptyTuple, N, M>
{
    type Result = Cons<V, Cons<Val, EmptyTuple, N, M>, Suc<N>, Suc<M>>;

    fn append(self, val: Val) -> Self::Result {
        let Cons(v, EmptyTuple, _) = self;
        Cons(v, Cons(val, EmptyTuple, PhantomData), PhantomData)
    }
}

impl<
        V: Debug,
        T: Tuple + Append<Val> + TupleEqual<EmptyTuple, Result = False>,
        Val: Debug,
        N: Nat + IsGreaterThan<One, Result = True>,
        M: Nat,
    > Append<Val> for Cons<V, T, N, M>
{
    type Result = Cons<V, <T as Append<Val>>::Result, N, M>;

    fn append(self, val: Val) -> Self::Result {
        let Cons(v, t, _) = self;
        Cons(v, t.append(val), PhantomData)
    }
}

pub trait ReverseTuple {
    type Reversed: Tuple;

    fn reverse(self) -> Self::Reversed;
}

impl ReverseTuple for EmptyTuple {
    type Reversed = EmptyTuple;

    fn reverse(self) -> Self::Reversed {
        EmptyTuple
    }
}

impl<V: Debug, N: Nat, M: Nat> ReverseTuple for Cons<V, EmptyTuple, N, M> {
    type Reversed = Cons<V, EmptyTuple, N, M>;

    fn reverse(self) -> Self::Reversed {
        self
    }
}

impl<
        V: Debug,
        T: Tuple + ReverseTuple + TupleEqual<EmptyTuple, Result = False>,
        N: Nat,
        M: Nat,
    > ReverseTuple for Cons<V, T, N, M>
where
    <T as ReverseTuple>::Reversed: Append<V>,
{
    type Reversed = <<T as ReverseTuple>::Reversed as Append<V>>::Result;

    fn reverse(self) -> Self::Reversed {
        let Cons(v, t, _) = self;
        t.reverse().append(v)
    }
}

// trait IsRecursivelyReversable {
//     type Result: Bool;
// }
//
// impl IsRecursivelyReversable for EmptyTuple {
//     type Result = True;
// }
//
// impl<V: Debug, T: Tuple, N: Nat, M: Nat> IsRecursivelyReversable for Cons<V, T, N, M> {
//     type Result = ;
// }

impl<A: Debug, M: Nat> From<A> for Cons<A, EmptyTuple, Suc<M>, M>
where
    A: Tupleness<M>,
    M: IsEqual<Zero, Result = True>,
{
    fn from(value: A) -> Self {
        Cons(value, EmptyTuple, PhantomData)
    }
}

impl<A: Debug, B: Debug, N: Nat, M: Nat, C: Debug, T: Tuple> From<(A, B)>
    for Cons<B, Cons<C, T, N, M>, Suc<N>, Suc<M>>
where
    (A, B): Tupleness<Suc<M>>,
    N: IsGreaterThan<Zero, Result = True>,
    Suc<M>: IsGreaterThan<Zero, Result = True>,
    Cons<C, T, N, M>: From<A>,
{
    fn from((a, b): (A, B)) -> Self {
        Cons(b, a.into(), PhantomData)
    }
}

pub trait Flatten {
    type Result;

    fn flatten(self) -> Self::Result;
}

impl<A: Debug, B: Debug, N1: Nat, N2: Nat, M1: Nat, M2: Nat> Flatten
    for Cons<A, Cons<B, EmptyTuple, N2, M2>, N1, M1>
{
    type Result = (A, B);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, _, _), _) = self;
        (a, b)
    }
}

impl<A: Debug, B: Debug, C: Debug, N1: Nat, N2: Nat, N3: Nat, M1: Nat, M2: Nat, M3: Nat> Flatten
    for Cons<A, Cons<B, Cons<C, EmptyTuple, N3, M3>, N2, M2>, N1, M1>
{
    type Result = (A, B, C);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, Cons(c, _, _), _), _) = self;
        (a, b, c)
    }
}

impl<
        A: Debug,
        B: Debug,
        C: Debug,
        D: Debug,
        N1: Nat,
        N2: Nat,
        N3: Nat,
        N4: Nat,
        M1: Nat,
        M2: Nat,
        M3: Nat,
        M4: Nat,
    > Flatten for Cons<A, Cons<B, Cons<C, Cons<D, EmptyTuple, N4, M4>, N3, M3>, N2, M2>, N1, M1>
{
    type Result = (A, B, C, D);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, Cons(c, Cons(d, _, _), _), _), _) = self;
        (a, b, c, d)
    }
}

impl<
        A: Debug,
        B: Debug,
        C: Debug,
        D: Debug,
        E: Debug,
        N1: Nat,
        N2: Nat,
        N3: Nat,
        N4: Nat,
        N5: Nat,
        M1: Nat,
        M2: Nat,
        M3: Nat,
        M4: Nat,
        M5: Nat,
    > Flatten
    for Cons<
        A,
        Cons<B, Cons<C, Cons<D, Cons<E, EmptyTuple, N5, M5>, N4, M4>, N3, M3>, N2, M2>,
        N1,
        M1,
    >
{
    type Result = (A, B, C, D, E);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, Cons(c, Cons(d, Cons(e, _, _), _), _), _), _) = self;
        (a, b, c, d, e)
    }
}

impl<
        A: Debug,
        B: Debug,
        C: Debug,
        D: Debug,
        E: Debug,
        F: Debug,
        N1: Nat,
        N2: Nat,
        N3: Nat,
        N4: Nat,
        N5: Nat,
        N6: Nat,
        M1: Nat,
        M2: Nat,
        M3: Nat,
        M4: Nat,
        M5: Nat,
        M6: Nat,
    > Flatten
    for Cons<
        A,
        Cons<
            B,
            Cons<C, Cons<D, Cons<E, Cons<F, EmptyTuple, N6, M6>, N5, M5>, N4, M4>, N3, M3>,
            N2,
            M2,
        >,
        N1,
        M1,
    >
{
    type Result = (A, B, C, D, E, F);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, Cons(c, Cons(d, Cons(e, Cons(f, _, _), _), _), _), _), _) = self;
        (a, b, c, d, e, f)
    }
}

impl<
        A: Debug,
        B: Debug,
        C: Debug,
        D: Debug,
        E: Debug,
        F: Debug,
        G: Debug,
        N1: Nat,
        N2: Nat,
        N3: Nat,
        N4: Nat,
        N5: Nat,
        N6: Nat,
        N7: Nat,
        M1: Nat,
        M2: Nat,
        M3: Nat,
        M4: Nat,
        M5: Nat,
        M6: Nat,
        M7: Nat,
    > Flatten
    for Cons<
        A,
        Cons<
            B,
            Cons<
                C,
                Cons<D, Cons<E, Cons<F, Cons<G, EmptyTuple, N7, M7>, N6, M6>, N5, M5>, N4, M4>,
                N3,
                M3,
            >,
            N2,
            M2,
        >,
        N1,
        M1,
    >
{
    type Result = (A, B, C, D, E, F, G);

    fn flatten(self) -> Self::Result {
        let Cons(a, Cons(b, Cons(c, Cons(d, Cons(e, Cons(f, Cons(g, _, _), _), _), _), _), _), _) =
            self;
        (a, b, c, d, e, f, g)
    }
}

impl<
        A: Debug,
        B: Debug,
        C: Debug,
        D: Debug,
        E: Debug,
        F: Debug,
        G: Debug,
        H: Debug,
        N1: Nat,
        N2: Nat,
        N3: Nat,
        N4: Nat,
        N5: Nat,
        N6: Nat,
        N7: Nat,
        N8: Nat,
        M1: Nat,
        M2: Nat,
        M3: Nat,
        M4: Nat,
        M5: Nat,
        M6: Nat,
        M7: Nat,
        M8: Nat,
    > Flatten
    for Cons<
        A,
        Cons<
            B,
            Cons<
                C,
                Cons<
                    D,
                    Cons<E, Cons<F, Cons<G, Cons<H, EmptyTuple, N8, M8>, N7, M7>, N6, M6>, N5, M5>,
                    N4,
                    M4,
                >,
                N3,
                M3,
            >,
            N2,
            M2,
        >,
        N1,
        M1,
    >
{
    type Result = (A, B, C, D, E, F, G, H);

    fn flatten(self) -> Self::Result {
        let Cons(
            a,
            Cons(b, Cons(c, Cons(d, Cons(e, Cons(f, Cons(g, Cons(h, _, _), _), _), _), _), _), _),
            _,
        ) = self;
        (a, b, c, d, e, f, g, h)
    }
}

pub struct ZippedValid<V, E, Level: Nat>(pub Result<V, ValidationError<E>>, pub PhantomData<Level>);

impl<Level: Nat, A, E> ZippedValid<A, E, Level>
where
    A: Tupleness<Level>,
{
    pub fn flatten_all<Cv: Debug, Ct: Tuple + ReverseTuple, Cm: Nat>(
        self,
    ) -> Valid<<<<Ct as ReverseTuple>::Reversed as Append<Cv>>::Result as Flatten>::Result, E>
    where
        Ct: TupleEqual<EmptyTuple, Result = False>,
        <Ct as ReverseTuple>::Reversed: Append<Cv>,
        Cons<Cv, Ct, Cm, Level>: From<A> + Flatten,
        <Cons<Cv, Ct, Level, Cm> as ReverseTuple>::Reversed: Flatten,
    {
        Valid(self.0.map(|v| {
            let tup: Cons<_, _, _, Level> = v.into();
            tup.reverse().flatten()
        }))
    }

    pub fn flatten<CustomLevel: Nat, Cv: Debug, Ct: Tuple + ReverseTuple, Cm: Nat>(
        self,
    ) -> Valid<<<<Ct as ReverseTuple>::Reversed as Append<Cv>>::Result as Flatten>::Result, E>
    where
        Ct: TupleEqual<EmptyTuple, Result = False>,
        <Ct as ReverseTuple>::Reversed: Append<Cv>,
        Cons<Cv, Ct, Cm, CustomLevel>: From<A> + Flatten,
        <Cons<Cv, Ct, CustomLevel, Cm> as ReverseTuple>::Reversed: Flatten,
    {
        Valid(self.0.map(|v| {
            let tup: Cons<_, _, _, CustomLevel> = v.into();
            tup.reverse().flatten()
        }))
    }

    pub fn zip<A1>(self, other: Valid<A1, E>) -> ZippedValid<(A, A1), E, Suc<Level>> {
        match self.0 {
            Ok(a) => match other.0 {
                Ok(a1) => ZippedValid(Ok((a, a1)), PhantomData),
                Err(e1) => ZippedValid(Err(e1), PhantomData),
            },
            Err(e1) => match other.0 {
                Ok(_) => ZippedValid(Err(e1), PhantomData),
                Err(e2) => ZippedValid(Err(e1.combine(e2)), PhantomData),
            },
        }
    }
}

// Copied implementation from Valid to make the ZippedValid compatible with the Valid interface
impl<A, E, Level: Nat> ZippedValid<A, E, Level> {
    pub fn map<A1>(self, f: impl FnOnce(A) -> A1) -> Valid<A1, E> {
        Valid(self.0.map(f))
    }

    pub fn foreach(self, mut f: impl FnMut(A)) -> Valid<A, E>
    where
        A: Clone,
    {
        match self.0 {
            Ok(a) => {
                f(a.clone());
                Valid::succeed(a)
            }
            Err(e) => Valid(Err(e)),
        }
    }

    pub fn succeed(a: A) -> Valid<A, E> {
        Valid(Ok(a))
    }

    pub fn is_succeed(&self) -> bool {
        self.0.is_ok()
    }

    pub fn and<A1>(self, other: Valid<A1, E>) -> Valid<A1, E> {
        Valid(self.0).zip(other).map(|(_, a1)| a1)
    }

    pub fn trace(self, message: &str) -> Valid<A, E> {
        let valid = self.0;
        if let Err(error) = valid {
            return Valid(Err(error.trace(message)));
        }

        Valid(valid)
    }

    pub fn fold<A1>(
        self,
        ok: impl FnOnce(A) -> Valid<A1, E>,
        err: impl FnOnce() -> Valid<A1, E>,
    ) -> Valid<A1, E> {
        match self.0 {
            Ok(a) => ok(a),
            Err(e) => Valid::<A1, E>(Err(e)).and(err()),
        }
    }

    pub fn from_iter<B>(
        iter: impl IntoIterator<Item = A>,
        f: impl Fn(A) -> Valid<B, E>,
    ) -> Valid<Vec<B>, E> {
        let mut values: Vec<B> = Vec::new();
        let mut errors: ValidationError<E> = ValidationError::empty();
        for a in iter.into_iter() {
            match f(a).to_result() {
                Ok(b) => {
                    values.push(b);
                }
                Err(err) => {
                    errors = errors.combine(err);
                }
            }
        }

        if errors.is_empty() {
            Valid::succeed(values)
        } else {
            Valid::from_validation_err(errors)
        }
    }

    pub fn from_option(option: Option<A>, e: E) -> Valid<A, E> {
        match option {
            Some(a) => Valid::succeed(a),
            None => Valid::fail(e),
        }
    }

    pub fn to_result(self) -> Result<A, ValidationError<E>> {
        self.0
    }

    pub fn and_then<B>(self, f: impl FnOnce(A) -> Valid<B, E>) -> Valid<B, E> {
        match self.0 {
            Ok(a) => f(a),
            Err(e) => Valid(Err(e)),
        }
    }

    pub fn unit(self) -> Valid<(), E> {
        self.map(|_| ())
    }

    pub fn some(self) -> Valid<Option<A>, E> {
        self.map(Some)
    }

    pub fn none() -> Valid<Option<A>, E> {
        Valid::succeed(None)
    }
    pub fn map_to<B>(self, b: B) -> Valid<B, E> {
        self.map(|_| b)
    }
    pub fn when(self, f: impl FnOnce() -> bool) -> Valid<(), E> {
        if f() {
            self.unit()
        } else {
            Valid::succeed(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::valid::{Two, Valid};

    #[test]
    fn test_flatten_all_3() {
        let valid1: Valid<_, String> = Valid::succeed(10);
        let valid2 = Valid::succeed(20);
        let valid3 = Valid::succeed(30);
        let zipped = valid1.zip2(valid2).zip(valid3);
        assert_eq!(Valid(Ok((10, 20, 30))), zipped.flatten_all());
    }

    #[test]
    fn test_flatten_all_4() {
        let valid1: Valid<_, String> = Valid::succeed(10);
        let valid2 = Valid::succeed(20);
        let valid3 = Valid::succeed(30);
        let valid4 = Valid::succeed(40);
        let zipped = valid1.zip2(valid2).zip(valid3).zip(valid4);
        assert_eq!(Valid(Ok((10, 20, 30, 40))), zipped.flatten_all());
    }

    #[test]
    fn test_flatten() {
        let result: Valid<_, String> = Valid::succeed(1)
            .zip2(Valid::succeed(2))
            .zip(Valid::succeed(3))
            .zip(Valid::succeed(4))
            .flatten::<Two, _, _, _>();
        assert_eq!(Valid(Ok(((1, 2), 3, 4))), result);
    }
}
