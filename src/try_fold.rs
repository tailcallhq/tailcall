use crate::valid::{Valid, Validator};

/// Trait for types that support a "try fold" operation.
///
/// `TryFolding` describes a composable folding operation that can potentially
/// fail. It can optionally consume an input to transform the provided value.
type TryFoldFn<'a, I, O, E> = Box<dyn Fn(&I, O) -> Valid<O, E> + 'a>;

pub struct TryFold<'a, I: 'a, O: 'a, E: 'a>(TryFoldFn<'a, I, O, E>);

impl<'a, I, O: Clone + 'a, E> TryFold<'a, I, O, E> {
    /// Try to fold the value with the input.
    ///
    /// # Parameters
    /// - `input`: The input used in the folding operation.
    /// - `value`: The value to be folded.
    ///
    /// # Returns
    /// Returns a `Valid` value, which can be either a success with the folded
    /// value or an error.
    pub fn try_fold(&self, input: &I, state: O) -> Valid<O, E> {
        (self.0)(input, state)
    }

    /// Combine two `TryFolding` implementors into a sequential operation.
    ///
    /// This method allows for chaining two `TryFolding` operations, where the
    /// result of the first operation (if successful) will be used as the
    /// input for the second operation.
    ///
    /// # Parameters
    /// - `other`: Another `TryFolding` implementor.
    ///
    /// # Returns
    /// Returns a combined `And` structure that represents the sequential
    /// folding operation.
    pub fn and(self, other: TryFold<'a, I, O, E>) -> Self {
        TryFold(Box::new(move |input, state| {
            self.try_fold(input, state.clone()).fold(
                |state| other.try_fold(input, state),
                || other.try_fold(input, state),
            )
        }))
    }

    /// Create a new `TryFold` with a specified folding function.
    ///
    /// # Parameters
    /// - `f`: The folding function.
    ///
    /// # Returns
    /// Returns a new `TryFold` instance.
    pub fn new(f: impl Fn(&I, O) -> Valid<O, E> + 'a) -> Self {
        TryFold(Box::new(f))
    }

    /// Transforms a TryFold<I, O, E> to TryFold<I, O1, E> by applying
    /// transformations. Check `transform_valid` if you want to return a
    /// `Valid` instead of an `O1`.
    ///
    /// # Parameters
    /// - `up`: A function that uses O and O1 to create a new O1.
    /// - `down`: A function that uses O1 to create a new O.
    ///
    /// # Returns
    /// Returns a new TryFold<I, O1, E> that applies the transformations.
    pub fn transform<O1: Clone>(
        self,
        up: impl Fn(O, O1) -> O1 + 'a,
        down: impl Fn(O1) -> O + 'a,
    ) -> TryFold<'a, I, O1, E> {
        self.transform_valid(
            move |o, o1| Valid::succeed(up(o, o1)),
            move |o1| Valid::succeed(down(o1)),
        )
    }

    /// Transforms a TryFold<I, O, E> to TryFold<I, O1, E> by applying
    /// transformations. Check `transform` if you want to return an `O1`
    /// instead of a `Valid`.
    ///
    /// # Parameters
    /// - `up`: A function that uses O and O1 to create a new Valid<O1, E>.
    /// - `down`: A function that uses O1 to create a new Valid<O, E>.
    ///
    /// # Returns
    /// Returns a new TryFold<I, O1, E> that applies the transformations.
    pub fn transform_valid<O1: Clone>(
        self,
        up: impl Fn(O, O1) -> Valid<O1, E> + 'a,
        down: impl Fn(O1) -> Valid<O, E> + 'a,
    ) -> TryFold<'a, I, O1, E> {
        TryFold(Box::new(move |i, o1| {
            down(o1.clone())
                .and_then(|o| self.try_fold(i, o))
                .and_then(|o| up(o, o1))
        }))
    }

    pub fn update(self, f: impl Fn(O) -> O + 'a) -> TryFold<'a, I, O, E> {
        self.transform(move |o, _| f(o), |o| o)
    }

    /// Create a `TryFold` that always succeeds with the provided state.
    ///
    /// # Parameters
    /// - `state`: The state to succeed with.
    ///
    /// # Returns
    /// Returns a `TryFold` that always succeeds with the provided state.
    pub fn succeed(f: impl Fn(&I, O) -> O + 'a) -> Self {
        TryFold(Box::new(move |i, o| Valid::succeed(f(i, o))))
    }

    /// Create a `TryFold` that doesn't do anything.
    ///
    /// # Returns
    /// Returns a `TryFold` that doesn't do anything.
    pub fn empty() -> Self {
        TryFold::new(|_, o| Valid::succeed(o))
    }

    /// Create a `TryFold` that always fails with the provided error.
    ///
    /// # Parameters
    /// - `e`: The error to fail with.
    ///
    /// # Returns
    /// Returns a `TryFold` that always fails with the provided error.
    pub fn fail(e: E) -> Self
    where
        E: Clone,
    {
        TryFold::new(move |_, _| Valid::fail(e.clone()))
    }

    /// Add trace logging to the fold operation.
    ///
    /// # Parameters
    ///
    /// * `msg` - The message to log when this fold operation is executed.
    ///
    /// # Returns
    ///
    /// Returns a new `TryFold` with trace logging added.
    pub fn trace(self, msg: &'a str) -> Self {
        TryFold::new(move |i, o| self.try_fold(i, o).trace(msg))
    }
}

impl<'a, I, O: Clone, E> FromIterator<TryFold<'a, I, O, E>> for TryFold<'a, I, O, E> {
    fn from_iter<T: IntoIterator<Item = TryFold<'a, I, O, E>>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let head = iter.next();

        if let Some(head) = head {
            head.and(TryFold::from_iter(iter))
        } else {
            TryFold::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::TryFold;
    use crate::valid::{Valid, ValidationError, Validator};

    #[test]
    fn test_and() {
        let t1 = TryFold::<i32, i32, ()>::new(|a: &i32, b: i32| Valid::succeed(a + b));
        let t2 = TryFold::<i32, i32, ()>::new(|a: &i32, b: i32| Valid::succeed(a * b));
        let t = t1.and(t2);

        let actual = t.try_fold(&2, 3).to_result().unwrap();
        let expected = 10;

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_one_failure() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::fail(a + b));
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a * b));
        let t = t1.and(t2);

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(5);

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_both_failure() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::fail(a + b));
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b));
        let t = t1.and(t2);

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(5).combine(ValidationError::new(6));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_order() {
        let calls = RefCell::new(Vec::new());
        let t1 = TryFold::<i32, i32, ()>::new(|a: &i32, b: i32| {
            calls.borrow_mut().push(1);
            Valid::succeed(a + b)
        }); // 2 + 3
        let t2 = TryFold::new(|a: &i32, b: i32| {
            calls.borrow_mut().push(2);
            Valid::succeed(a * b)
        }); // 2 * 3
        let t3 = TryFold::new(|a: &i32, b: i32| {
            calls.borrow_mut().push(3);
            Valid::succeed(a * b * 100)
        }); // 2 * 6
        let _t = t1.and(t2).and(t3).try_fold(&2, 3);

        assert_eq!(*calls.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_1_3_failure_left() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::fail(a + b)); // 2 + 3
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a * b)); // 2 * 3
        let t3 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b * 100)); // 2 * 6
        let t = t1.and(t2).and(t3);

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(5).combine(ValidationError::new(600));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_1_3_failure_right() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::fail(a + b)); // 2 + 3
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a * b)); // 2 * 3
        let t3 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b * 100)); // 2 * 6
        let t = t1.and(t2.and(t3));

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(5).combine(ValidationError::new(1200));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_2_3_failure() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a + b));
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b));
        let t3 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b * 100));
        let t = t1.and(t2.and(t3));

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(10).combine(ValidationError::new(1000));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_try_all() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a + b));
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b));
        let t3 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b * 100));
        let t = TryFold::from_iter(vec![t1, t2, t3]);

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(10).combine(ValidationError::new(1000));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_try_all_1_3_fail() {
        let t1 = TryFold::new(|a: &i32, b: i32| Valid::fail(a + b));
        let t2 = TryFold::new(|a: &i32, b: i32| Valid::succeed(a * b));
        let t3 = TryFold::new(|a: &i32, b: i32| Valid::fail(a * b * 100));
        let t = TryFold::from_iter(vec![t1, t2, t3]);

        let actual = t.try_fold(&2, 3).to_result().unwrap_err();
        let expected = ValidationError::new(5).combine(ValidationError::new(1200));

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_transform() {
        let t: TryFold<'_, i32, String, ()> = TryFold::succeed(|a: &i32, b: i32| a + b).transform(
            |v: i32, _| v.to_string(),
            |v: String| v.parse::<i32>().unwrap(),
        );

        let actual = t.try_fold(&2, "3".to_string()).to_result().unwrap();
        let expected = "5".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_transform_valid() {
        let t: TryFold<'_, i32, String, ()> = TryFold::succeed(|a: &i32, b: i32| a + b)
            .transform_valid(
                |v: i32, _| Valid::succeed(v.to_string()),
                |v: String| Valid::succeed(v.parse::<i32>().unwrap()),
            );

        let actual = t.try_fold(&2, "3".to_string()).to_result().unwrap();
        let expected = "5".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_update() {
        let t = TryFold::<i32, i32, String>::succeed(|a: &i32, b: i32| a + b).update(|a| a + 1);
        let actual = t.try_fold(&2, 3).to_result().unwrap();
        let expected = 6;
        assert_eq!(actual, expected);
    }
}
