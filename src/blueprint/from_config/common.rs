use crate::blueprint::Type;

pub(crate) trait TypeLike {
  fn name(&self) -> &str;
  fn list(&self) -> bool;
  fn non_null(&self) -> bool;
  fn list_type_required(&self) -> bool;
}

pub(crate) fn to_type<T>(field: &T, override_non_null: Option<bool>) -> Type
where
  T: TypeLike,
{
  let name = field.name();
  let list = field.list();
  let list_type_required = field.list_type_required();
  let non_null = if let Some(non_null) = override_non_null {
    non_null
  } else {
    field.non_null()
  };

  if list {
    Type::ListType {
      of_type: Box::new(Type::NamedType { name: name.to_string(), non_null: list_type_required }),
      non_null,
    }
  } else {
    Type::NamedType { name: name.to_string(), non_null }
  }
}
