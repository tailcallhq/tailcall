use tailcall_valid::Valid;

use crate::core::merge_right::MergeRight;
use crate::core::primitive::Primitive;

/// A trait representing types that are **invariant** with respect to merging
/// operations.
///
/// In type theory, an invariant type is neither covariant nor contravariant.
/// When merging two values of an invariant type, the result does not expand or
/// shrink in terms of its constraints or possibilities. Instead, merging
/// typically follows custom logic to unify the two values into one, preserving
/// their essential properties without introducing additional flexibility or
/// restrictions.
///
/// The `unify` method defines how two values of the type can be combined while
/// maintaining their invariance. This is useful in scenarios where a type must
/// strictly adhere to certain constraints, and any merging must respect those
/// constraints.
pub trait Invariant: Sized {
    fn unify(self, other: Self) -> Valid<Self, String>;
}

/// A trait representing types that exhibit **contravariant** behavior during
/// merging operations.
///
/// In type theory, a contravariant type allows substitution with more general
/// (less specific) types. In the context of merging, a contravariant type can
/// "shrink" when combined with another value, resulting in a type that is more
/// restrictive or specific than either of the original types.
///
/// The `shrink` method defines how two values of the type can be merged into a
/// new value that represents a narrower scope or more specific constraints.
/// This is useful in scenarios where combining two values should result in the
/// most restrictive interpretation.
pub trait Contravariant: Sized {
    fn shrink(self, other: Self) -> Valid<Self, String>;
}

/// A trait representing types that exhibit **covariant** behavior during
/// merging operations.
///
/// In type theory, a covariant type allows substitution with more specific
/// (less general) types. In the context of merging, a covariant type can
/// "expand" when combined with another value, resulting in a type that is less
/// restrictive or more general than either of the original types.
///
/// The `expand` method defines how two values of the type can be merged into a
/// new value that represents a broader scope or fewer constraints. This is
/// useful in scenarios where combining two values should result in the most
/// permissive interpretation.
pub trait Covariant: Sized {
    fn expand(self, other: Self) -> Valid<Self, String>;
}

/// Implements the `Invariant` trait for all types that implement `Primitive`.
///
/// This implementation defines how two primitive values can be unified without
/// changing their invariance. The `unify` method uses the `merge_right`
/// function to combine `self` and `other`, preserving the essential properties
/// of the type.
impl<A: Primitive + Sized> Invariant for A {
    fn unify(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}
