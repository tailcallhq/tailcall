use crate::valid::{Valid, ValidExtensions, ValidationError};

type Transformation<S, T, E> = dyn FnOnce(&S, T) -> Valid<T, E>;
pub struct Transform<S, T, E = &'static str> {
  pub transform: Box<Transformation<S, T, E>>,
}

impl<S, T, E> Transform<S, T, E> {
  pub fn new(transform: impl FnOnce(&S, T) -> Valid<T, E> + 'static) -> Self {
    Self { transform: Box::new(transform) }
  }

  pub fn combine(self, other: Self) -> Self
  where
    T: Clone + 'static,
    S: 'static,
    E: 'static,
  {
    Self::new(move |s, t| match (self.transform)(s, t.clone()) {
      Ok(blueprint) => (other.transform)(s, blueprint),
      Err(e) => Err::<T, ValidationError<E>>(e).validate_or((other.transform)(s, t)),
    })
  }

  pub fn transform(self, s: &S, t: T) -> Valid<T, E> {
    (self.transform)(s, t)
  }
}

impl<S, T: Clone, E> std::ops::Add for Transform<S, T, E>
where
  T: Clone + 'static,
  S: 'static,
  E: 'static,
{
  type Output = Self;
  fn add(self, rhs: Self) -> Self::Output {
    self.combine(rhs)
  }
}

#[cfg(test)]
mod tests {
  use crate::blueprint::transform::Transform;

  #[test]
  fn test_combine() {
    let t1 = Transform::<i32, i32>::new(|a, b| Ok(a + b));
    let t2 = Transform::<i32, i32>::new(|a, b| Ok(a * b));
    let t = t1 + t2;

    let actual = t.transform(&2, 3).unwrap();
    let expected = 10;

    assert_eq!(actual, expected)
  }
}
