use std::fmt::Display;
use std::rc::Rc;
use std::sync::Arc;

use jaq_core::val::Range;
use jaq_core::{Error, Exn, ValR};
use jaq_json::Val;

use crate::core::ir::{EvalContext, ResolverContextLike};
use crate::core::path::{PathString, PathValue, ValueString};

#[derive(Clone)]
/// Used to hold a PathJqValue struct or the JQ Val struct
pub enum PathValueEnum<'a> {
    PathValue(Arc<&'a dyn PathJqValue>),
    Val(Val),
}

impl jaq_std::ValT for PathValueEnum<'_> {
    fn into_seq<S: FromIterator<Self>>(self) -> Result<S, Self> {
        // convert an array into a sequence
        match self {
            PathValueEnum::PathValue(_) => Err(self),
            PathValueEnum::Val(val) => match val {
                Val::Arr(a) => match Rc::try_unwrap(a) {
                    Ok(a) => Ok(a.into_iter().map(Self::Val).collect()),
                    Err(a) => Ok(a.iter().cloned().map(Self::Val).collect()),
                },
                _ => Err(Self::Val(val)),
            },
        }
    }

    fn as_isize(&self) -> Option<isize> {
        match self {
            PathValueEnum::PathValue(_) => None,
            PathValueEnum::Val(val) => val.as_isize(),
        }
    }

    fn as_f64(&self) -> Result<f64, Error<Self>> {
        match self {
            PathValueEnum::PathValue(_) => Err(Error::new(Self::Val(Val::from(
                "Cannot convert context to f64".to_string(),
            )))),
            PathValueEnum::Val(val) => match val.as_f64() {
                Ok(val) => Ok(val),
                Err(err) => {
                    let val = err.into_val();
                    Err(Error::new(Self::Val(val)))
                }
            },
        }
    }
}

impl jaq_core::ValT for PathValueEnum<'_> {
    fn from_num(n: &str) -> ValR<Self> {
        match Val::from_num(n) {
            Ok(val) => ValR::Ok(Self::Val(val)),
            Err(err) => {
                let val = err.into_val();
                Err(Error::new(Self::Val(val)))
            }
        }
    }

    fn from_map<I: IntoIterator<Item = (Self, Self)>>(iter: I) -> ValR<Self> {
        let result: Result<Vec<(Val, Val)>, String> = iter
            .into_iter()
            .map(|(k, v)| match (k, v) {
                (PathValueEnum::Val(key), PathValueEnum::Val(value)) => Ok((key, value)),
                _ => Err("Invalid key or value type for map".into()),
            })
            .collect();

        match result {
            Ok(pairs) => match Val::from_map(pairs) {
                Ok(val) => ValR::Ok(PathValueEnum::Val(val)),
                Err(err) => {
                    let val = err.into_val();
                    Err(Error::new(Self::Val(val)))
                }
            },
            Err(e) => Err(Error::new(Self::Val(Val::from(e)))),
        }
    }

    fn values(self) -> Box<dyn Iterator<Item = ValR<Self>>> {
        match self {
            PathValueEnum::PathValue(_) => panic!("Cannot iterate context"),
            PathValueEnum::Val(val) => Box::new(val.values().map(|v| {
                v.map(PathValueEnum::Val).map_err(|err| {
                    let val = err.into_val();
                    Error::new(PathValueEnum::Val(val))
                })
            })),
        }
    }

    fn index(self, index: &Self) -> ValR<Self> {
        let PathValueEnum::Val(index) = index else {
            return ValR::Err(Error::new(Self::Val(Val::from(format!(
                "Could not convert index `{}` val.",
                index
            )))));
        };

        match self {
            PathValueEnum::PathValue(pv) => {
                let Some(v) = pv.get_value(index) else {
                    return ValR::Err(Error::new(Self::Val(Val::from(format!(
                        "Could not find key `{}` in context.",
                        index
                    )))));
                };

                match v {
                    crate::core::path::ValueString::Value(cow) => {
                        let cv = cow.as_ref().clone();
                        match cv.into_json() {
                            Ok(js) => Ok(Self::Val(Val::from(js))),
                            Err(err) => ValR::Err(Error::new(Self::Val(Val::from(format!(
                                "Could not convert value to json: {:?}",
                                err
                            ))))),
                        }
                    }
                    crate::core::path::ValueString::String(cow) => {
                        let v = cow.to_string();
                        Ok(Self::Val(Val::from(v)))
                    }
                }
            }
            PathValueEnum::Val(val) => match val.index(index) {
                Ok(val) => ValR::Ok(Self::Val(val)),
                Err(err) => {
                    let val = err.into_val();
                    Err(Error::new(Self::Val(val)))
                }
            },
        }
    }

    fn range(self, range: jaq_core::val::Range<&Self>) -> ValR<Self> {
        let (start, end) = (
            range
                .start
                .map(|v| match v {
                    PathValueEnum::PathValue(_) => ValR::Err(Error::new(Val::from(
                        "Could not convert range start to val.".to_string(),
                    ))),
                    PathValueEnum::Val(val) => Ok(val.clone()),
                })
                .transpose(),
            range
                .end
                .map(|v| match v {
                    PathValueEnum::PathValue(_) => ValR::Err(Error::new(Val::from(
                        "Could not convert range end to val.".to_string(),
                    ))),
                    PathValueEnum::Val(val) => Ok(val.clone()),
                })
                .transpose(),
        );

        let (start, end) = match (start, end) {
            (Ok(start), Ok(end)) => (start, end),
            (Ok(_), Err(err)) => {
                let val = err.into_val();
                return Err(Error::new(Self::Val(val)));
            }
            (Err(err), Ok(_)) => {
                let val = err.into_val();
                return Err(Error::new(Self::Val(val)));
            }
            (Err(_), Err(_)) => {
                return ValR::Err(Error::new(Self::Val(Val::from(
                    "Could not convert range to val.".to_string(),
                ))))
            }
        };

        let range = Range { start: start.as_ref(), end: end.as_ref() };

        match self {
            PathValueEnum::PathValue(_) => ValR::Err(Error::new(Self::Val(Val::from(
                "Cannot apply range operation at the context".to_string(),
            )))),
            PathValueEnum::Val(val) => match val.range(range) {
                Ok(val) => ValR::Ok(Self::Val(val)),
                Err(err) => {
                    let val = err.into_val();
                    Err(Error::new(Self::Val(val)))
                }
            },
        }
    }

    fn map_values<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        let f_new = move |x: Val| -> _ {
            let iter = f(Self::Val(x));
            iter.map(|v| match v {
                Ok(enum_val) => match enum_val {
                    PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(
                        Val::from("Cannot convert context to val.".to_string()),
                    ))),
                    PathValueEnum::Val(val) => Ok(val),
                },
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Val::from(format!(
                    "Function execution failed with: {:?}",
                    err
                ))))),
            })
        };

        match self {
            PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(
                Val::from("Cannot apply map_values operation at the context".to_string()),
            )))),
            PathValueEnum::Val(val) => match val.map_values(opt, f_new) {
                Ok(val) => jaq_core::ValX::Ok(Self::Val(val)),
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(Val::from(
                    format!("The map_values failed because: {:?}", err),
                ))))),
            },
        }
    }

    fn map_index<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        index: &Self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        let PathValueEnum::Val(index) = index else {
            return jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(Val::from(format!(
                "Could not convert index `{}` val.",
                index
            ))))));
        };

        let f_new = move |x: Val| -> _ {
            let iter = f(Self::Val(x));
            iter.map(|v| match v {
                Ok(enum_val) => match enum_val {
                    PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(
                        Val::from("Cannot convert context to val.".to_string()),
                    ))),
                    PathValueEnum::Val(val) => Ok(val),
                },
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Val::from(format!(
                    "Function execution failed with: {:?}",
                    err
                ))))),
            })
        };

        match self {
            PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(
                Val::from("Cannot apply map_index operation at the context".to_string()),
            )))),
            PathValueEnum::Val(val) => match val.map_index(index, opt, f_new) {
                Ok(val) => jaq_core::ValX::Ok(Self::Val(val)),
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(Val::from(
                    format!("The map_index failed because: {:?}", err),
                ))))),
            },
        }
    }

    fn map_range<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        range: jaq_core::val::Range<&Self>,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        let (start, end) = (
            range
                .start
                .map(|v| match v {
                    PathValueEnum::PathValue(_) => ValR::Err(Error::new(Val::from(
                        "Could not convert range start to val.".to_string(),
                    ))),
                    PathValueEnum::Val(val) => Ok(val.clone()),
                })
                .transpose(),
            range
                .end
                .map(|v| match v {
                    PathValueEnum::PathValue(_) => ValR::Err(Error::new(Val::from(
                        "Could not convert range end to val.".to_string(),
                    ))),
                    PathValueEnum::Val(val) => Ok(val.clone()),
                })
                .transpose(),
        );

        let (start, end) = match (start, end) {
            (Ok(start), Ok(end)) => (start, end),
            (Ok(_), Err(err)) => {
                let val = err.into_val();
                return Err(Exn::from(Error::new(Self::Val(val))));
            }
            (Err(err), Ok(_)) => {
                let val = err.into_val();
                return Err(Exn::from(Error::new(Self::Val(val))));
            }
            (Err(_), Err(_)) => {
                return Err(Exn::from(Error::new(Self::Val(Val::from(
                    "Could not convert range to val.".to_string(),
                )))))
            }
        };

        let range = Range { start: start.as_ref(), end: end.as_ref() };

        let f_new = move |x: Val| -> _ {
            let iter = f(Self::Val(x));
            iter.map(|v| match v {
                Ok(enum_val) => match enum_val {
                    PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(
                        Val::from("Cannot convert context to val.".to_string()),
                    ))),
                    PathValueEnum::Val(val) => Ok(val),
                },
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Val::from(format!(
                    "Function execution failed with: {:?}",
                    err
                ))))),
            })
        };

        match self {
            PathValueEnum::PathValue(_) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(
                Val::from("Cannot apply map_range operation at the context".to_string()),
            )))),
            PathValueEnum::Val(val) => match val.map_range(range, opt, f_new) {
                Ok(val) => jaq_core::ValX::Ok(Self::Val(val)),
                Err(err) => jaq_core::ValX::Err(Exn::from(Error::new(Self::Val(Val::from(
                    format!("The map_range failed because: {:?}", err),
                ))))),
            },
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            PathValueEnum::PathValue(_) => true,
            PathValueEnum::Val(val) => val.as_bool(),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            PathValueEnum::PathValue(_) => Some("[Context]"),
            PathValueEnum::Val(val) => val.as_str(),
        }
    }
}

impl<'a> FromIterator<PathValueEnum<'a>> for PathValueEnum<'a> {
    fn from_iter<I: IntoIterator<Item = PathValueEnum<'a>>>(iter: I) -> Self {
        let iter = iter.into_iter().filter_map(|v| match v {
            PathValueEnum::PathValue(_) => None,
            PathValueEnum::Val(val) => Some(val),
        });
        Self::Val(Val::from_iter(iter))
    }
}

impl std::ops::Add for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PathValueEnum::Val(self_val), PathValueEnum::Val(rhs_val)) => {
                match self_val.add(rhs_val) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    }
                }
            }
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform add operation with context.".to_string(),
            )))),
        }
    }
}

impl std::ops::Sub for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PathValueEnum::Val(self_val), PathValueEnum::Val(rhs_val)) => {
                match self_val.sub(rhs_val) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    }
                }
            }
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform sub operation with context.".to_string(),
            )))),
        }
    }
}

impl std::ops::Mul for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PathValueEnum::Val(self_val), PathValueEnum::Val(rhs_val)) => {
                match self_val.mul(rhs_val) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    }
                }
            }
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform mul operation with context.".to_string(),
            )))),
        }
    }
}

impl std::ops::Div for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PathValueEnum::Val(self_val), PathValueEnum::Val(rhs_val)) => {
                match self_val.div(rhs_val) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    }
                }
            }
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform div operation with context.".to_string(),
            )))),
        }
    }
}

impl std::ops::Rem for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PathValueEnum::Val(self_val), PathValueEnum::Val(rhs_val)) => {
                match self_val.rem(rhs_val) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    }
                }
            }
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform rem operation with context.".to_string(),
            )))),
        }
    }
}

impl std::ops::Neg for PathValueEnum<'_> {
    type Output = ValR<Self>;

    fn neg(self) -> Self::Output {
        match self {
            PathValueEnum::Val(self_val) => match self_val.neg() {
                Ok(val) => ValR::Ok(Self::Val(val)),
                Err(err) => {
                    let val = err.into_val();
                    Err(Error::new(Self::Val(val)))
                }
            },
            _ => ValR::Err(Error::new(PathValueEnum::Val(Val::from(
                "Cannot perform neg operation at context.".to_string(),
            )))),
        }
    }
}

impl Display for PathValueEnum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathValueEnum::PathValue(_) => "[Context]".to_string().fmt(f),
            PathValueEnum::Val(val) => val.fmt(f),
        }
    }
}

impl std::fmt::Debug for PathValueEnum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathValueEnum::PathValue(_) => derive_more::Debug::fmt(&"[Context]".to_string(), f),
            PathValueEnum::Val(val) => derive_more::Debug::fmt(&val, f),
        }
    }
}

impl PartialEq for PathValueEnum<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathValueEnum::PathValue(_), PathValueEnum::PathValue(_)) => true,
            (PathValueEnum::PathValue(_), PathValueEnum::Val(_)) => false,
            (PathValueEnum::Val(_), PathValueEnum::PathValue(_)) => false,
            (PathValueEnum::Val(self_val), PathValueEnum::Val(other_val)) => self_val.eq(other_val),
        }
    }
}

impl PartialOrd for PathValueEnum<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PathValueEnum<'_> {}

impl Ord for PathValueEnum<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (PathValueEnum::PathValue(_), PathValueEnum::PathValue(_)) => std::cmp::Ordering::Equal,
            (PathValueEnum::PathValue(_), PathValueEnum::Val(_)) => std::cmp::Ordering::Greater,
            (PathValueEnum::Val(_), PathValueEnum::PathValue(_)) => std::cmp::Ordering::Less,
            (PathValueEnum::Val(self_val), PathValueEnum::Val(other_val)) => {
                self_val.cmp(other_val)
            }
        }
    }
}

impl From<f64> for PathValueEnum<'_> {
    fn from(value: f64) -> Self {
        Self::Val(Val::from(value))
    }
}

impl From<PathValueEnum<'_>> for serde_json::Value {
    fn from(value: PathValueEnum<'_>) -> Self {
        match value {
            PathValueEnum::PathValue(_) => serde_json::Value::String("[Context]".to_string()),
            PathValueEnum::Val(val) => serde_json::Value::from(val),
        }
    }
}

impl From<String> for PathValueEnum<'_> {
    fn from(value: String) -> Self {
        Self::Val(Val::from(value))
    }
}

impl From<isize> for PathValueEnum<'_> {
    fn from(value: isize) -> Self {
        Self::Val(Val::from(value))
    }
}

impl From<bool> for PathValueEnum<'_> {
    fn from(value: bool) -> Self {
        Self::Val(Val::from(value))
    }
}

/// Used to get get keys/index out of json compatible objects like EvalContext
pub trait PathJqValue {
    fn get_value<'a>(&'a self, index: &Val) -> Option<ValueString<'a>>;
    fn get_values<'a>(&'a self, index: &[&str]) -> Option<ValueString<'a>>;
}

impl<Ctx: ResolverContextLike> PathJqValue for EvalContext<'_, Ctx> {
    fn get_value<'a>(&'a self, index: &Val) -> Option<ValueString<'a>> {
        let Val::Str(index) = index else { return None };
        self.raw_value(&[index.as_str()])
    }

    fn get_values<'a>(&'a self, path: &[&str]) -> Option<ValueString<'a>> {
        self.raw_value(path)
    }
}

impl PathJqValue for serde_json::Value {
    fn get_value(&self, index: &Val) -> Option<ValueString<'_>> {
        match self {
            serde_json::Value::Object(map) => {
                let Val::Str(index) = index else { return None };
                map.get(index.as_str()).map(|v| {
                    ValueString::Value(std::borrow::Cow::Owned(
                        async_graphql_value::ConstValue::from_json(v.clone()).unwrap(),
                    ))
                })
            }
            serde_json::Value::Array(list) => {
                let Val::Int(index) = index else { return None };
                list.get(*index as usize).map(|v| {
                    ValueString::Value(std::borrow::Cow::Owned(
                        async_graphql_value::ConstValue::from_json(v.clone()).unwrap(),
                    ))
                })
            }
            _ => None,
        }
    }

    fn get_values<'a>(&'a self, _index: &[&str]) -> Option<ValueString<'a>> {
        todo!()
    }
}

/// Used as a type parameter to accept objects that implement both traits
pub trait PathJqValueString: PathString + PathJqValue + PathValue {}

impl<Ctx: ResolverContextLike> PathJqValueString for EvalContext<'_, Ctx> {}

impl PathJqValueString for serde_json::Value {}
