use super::transform::Transform;

struct UnusedTypes {}

impl Transform for UnusedTypes {
  fn transform(_blueprint: super::Blueprint) -> crate::valid::Valid<super::Blueprint, &'static str> {
    todo!()
  }
}
