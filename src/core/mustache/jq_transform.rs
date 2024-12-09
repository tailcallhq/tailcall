use std::fmt::Display;
use std::sync::{Arc, RwLock};

use jaq_core::load::parse::Term;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::val::Range;
use jaq_core::{Compiler, Ctx, Error, Exn, Filter, Native, RcIter, ValR};
use jaq_json::Val;
use lazy_static::lazy_static;

use crate::core::ir::{EvalContext, ResolverContextLike};
use crate::core::json::JsonLike;
use crate::core::path::{PathString, PathValue, ValueString};

lazy_static! {
    static ref JQ_TEMPLATE_STORAGE: RwLock<Vec<Filter<Native<PathValueEnum<'static>>>>> =
        RwLock::new(Vec::new());
}

/// Used to represent a JQ template. Currently used only on @expr directive.
#[derive(Clone)]
pub struct JqTransform {
    /// The compiled template transformation
    template_id: usize,
    /// The IR representation, used for debug purposes
    representation: String,
}

#[derive(Clone)]
pub enum PathValueEnum<'a> {
    PathValue(Arc<&'a dyn PathJqValue>),
    Val(Val),
}

pub trait PathJqValue {
    fn get_value<'a>(&'a self, index: &Val) -> Option<ValueString<'a>>;
}

impl<Ctx: ResolverContextLike> PathJqValue for EvalContext<'_, Ctx> {
    fn get_value<'a>(&'a self, index: &Val) -> Option<ValueString<'a>> {
        let Val::Str(index) = index else { return None };
        self.raw_value(&[index.as_str()])
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
}
pub trait PathJqValueString: PathString + PathJqValue {}

impl<Ctx: ResolverContextLike> PathJqValueString for EvalContext<'_, Ctx> {}

impl PathJqValueString for serde_json::Value {}

impl jaq_std::ValT for PathValueEnum<'_> {
    fn into_seq<S: FromIterator<Self>>(self) -> Result<S, Self> {
        todo!()
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
        match (self, other) {
            (PathValueEnum::PathValue(_), PathValueEnum::PathValue(_)) => None,
            (PathValueEnum::PathValue(_), PathValueEnum::Val(_)) => None,
            (PathValueEnum::Val(_), PathValueEnum::PathValue(_)) => None,
            (PathValueEnum::Val(self_val), PathValueEnum::Val(other_val)) => {
                self_val.partial_cmp(other_val)
            }
        }
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

impl JqTransform {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &str) -> Result<Self, JqTemplateError> {
        // the term is used because it can be easily serialized, deserialized and hashed
        let term = Self::parse_template(template);

        // calculate if the expression can be replaced with mustache
        let is_mustache = Self::recursive_is_mustache(&term);
        if is_mustache {
            return Err(JqTemplateError::JqIsMustache);
        }

        // calculate if the expression returns always a constant value
        let is_const = Self::calculate_is_const(&term);
        if is_const {
            return Err(JqTemplateError::JqIstConst);
        }

        // the template is used to be parsed in to the IR AST
        let template = File { code: template, path: () };
        // defs is used to extend the syntax with custom definitions of functions, like
        // 'toString'
        let defs = jaq_std::defs();
        // the loader is used to load custom modules
        let loader = Loader::new(defs);
        // the arena is used to keep the loaded modules
        let arena = Arena::default();
        // load the modules
        let modules = loader.load(&arena, template).map_err(|errs| {
            JqTemplateError::JqLoadError(errs.into_iter().map(|e| format!("{:?}", e.1)).collect())
        })?;

        // the AST of the operation, used to transform the data
        let filter = Compiler::<_, Native<PathValueEnum>>::default()
            .with_funs(jaq_std::funs())
            .compile(modules)
            .map_err(|errs| {
                JqTemplateError::JqCompileError(
                    errs.into_iter().map(|e| format!("{:?}", e.1)).collect(),
                )
            })?;

        let mut write_lock = JQ_TEMPLATE_STORAGE.write().unwrap();

        let template_id = write_lock.len();
        let filter = filter;
        write_lock.push(filter);

        Ok(Self { template_id, representation: format!("{:?}", term) })
    }

    /// Used to execute the transformation of the JqTemplate
    pub fn run<'input>(&self, data: PathValueEnum<'input>) -> Vec<ValR<PathValueEnum<'input>>> {
        let inputs = RcIter::new(core::iter::empty());
        let ctx = Ctx::new([], &inputs);

        let read_guard = JQ_TEMPLATE_STORAGE.read().unwrap();

        let filter: &Filter<Native<PathValueEnum<'input>>> =
            unsafe { std::mem::transmute(read_guard.get(self.template_id).unwrap()) };

        filter.run((ctx, data)).collect::<Vec<_>>()
    }

    /// Used to calculate the result and return it as json
    pub fn render_value(&self, value: PathValueEnum<'_>) -> async_graphql_value::ConstValue {
        let res = self.run(value);
        let res: Vec<async_graphql_value::ConstValue> = res
            .into_iter()
            // TODO: handle error correct, now we ignore it
            .filter_map(|v| match v {
                Ok(v) => Some(v),
                Err(err) => {
                    println!("ERR: {:?}", err);
                    None
                }
            })
            .map(std::convert::Into::into)
            .map(async_graphql_value::ConstValue::from_json)
            // TODO: handle error correct, now we ignore it
            .filter_map(|v| match v {
                Ok(v) => Some(v),
                Err(err) => {
                    println!("ERR: {:?}", err);
                    None
                }
            })
            .collect();
        let res_len = res.len();
        if res_len == 0 {
            async_graphql_value::ConstValue::Null
        } else if res_len == 1 {
            res.into_iter().next().unwrap()
        } else {
            async_graphql_value::ConstValue::array(res)
        }
    }

    /// Used to parse the template string and return the IR representation
    fn parse_template(template: &str) -> Term<&str> {
        let lexer = jaq_core::load::Lexer::new(template);
        let lex = lexer.lex().unwrap_or_default();
        let mut parser = jaq_core::load::parse::Parser::new(&lex);
        parser.term().unwrap_or_default()
    }

    /// Used as a helper function to determine if the term can be supported with
    /// Mustache implementation
    fn recursive_is_mustache(term: &Term<&str>) -> bool {
        match term {
            Term::Id => true,
            Term::Recurse => false,
            // const number values
            Term::Num(_) => true,
            // const string values
            Term::Str(formater, inner) => formater.is_none() && (inner.len() == 1),
            Term::Arr(_) => false,
            Term::Obj(_) => false,
            Term::Neg(_) => false,
            Term::Pipe(_, _, _) => false,
            Term::BinOp(_, _, _) => false,
            Term::Label(_, _) => false,
            Term::Break(_) => false,
            Term::Fold(_, _, _, _) => false,
            Term::TryCatch(_, _) => false,
            Term::IfThenElse(_, _) => false,
            Term::Def(_, _) => false,
            // 'true' and 'false' values
            Term::Call(name, args) => (*name == "true" || *name == "false") && args.is_empty(),
            Term::Var(_) => false,
            // paths .data.foo.bar
            Term::Path(local_term, path) => {
                Self::recursive_is_mustache(local_term) && Self::is_path_select_operation(path)
            }
        }
    }

    /// Used to check if a JQ path can be supported by mustache
    fn is_path_mustache(term: &Term<&str>) -> bool {
        match term {
            Term::Id => true,
            Term::Recurse => false,
            // numbers, for example: .[1]
            Term::Num(_) => false,
            // string, for example: .data.user
            Term::Str(formater, inner) => formater.is_none() && (inner.len() == 1),
            Term::Arr(_) => false,
            Term::Obj(_) => false,
            Term::Neg(_) => false,
            Term::Pipe(_, _, _) => false,
            Term::BinOp(_, _, _) => false,
            Term::Label(_, _) => false,
            Term::Break(_) => false,
            Term::Fold(_, _, _, _) => false,
            Term::TryCatch(_, _) => false,
            Term::IfThenElse(_, _) => false,
            Term::Def(_, _) => false,
            // 'true' and 'false' values
            Term::Call(_, _) => false,
            Term::Var(_) => false,
            // paths .data.foo.bar
            Term::Path(local_term, path) => {
                Self::is_path_mustache(local_term) && Self::is_path_select_operation(path)
            }
        }
    }

    /// Used to check if the path indicates a select operation or modify
    fn is_path_select_operation(path: &jaq_core::path::Path<Term<&str>>) -> bool {
        path.0.iter().all(|part| match part {
            (jaq_core::path::Part::Index(_), jaq_core::path::Opt::Optional) => false,
            (jaq_core::path::Part::Index(idx), jaq_core::path::Opt::Essential) => {
                Self::is_path_mustache(idx)
            }
            (jaq_core::path::Part::Range(_, _), jaq_core::path::Opt::Optional) => false,
            (jaq_core::path::Part::Range(_, _), jaq_core::path::Opt::Essential) => false,
        })
    }

    /// Used to calcuate if the template always returns a constant value
    fn calculate_is_const(term: &Term<&str>) -> bool {
        match term {
            Term::Id => false,
            Term::Recurse => false,
            // const number values
            Term::Num(_) => true,
            // const string values
            Term::Str(formater, inner) => formater.is_none() && (inner.len() == 1),
            Term::Arr(_) => false,
            Term::Obj(_) => false,
            Term::Neg(_) => false,
            Term::Pipe(_, _, _) => false,
            Term::BinOp(_, _, _) => false,
            Term::Label(_, _) => false,
            Term::Break(_) => false,
            Term::Fold(_, _, _, _) => false,
            Term::TryCatch(_, _) => false,
            Term::IfThenElse(_, _) => false,
            Term::Def(_, _) => false,
            // 'true' and 'false' values
            Term::Call(name, args) => (*name == "true" || *name == "false") && args.is_empty(),
            Term::Var(_) => false,
            Term::Path(_, _) => false,
        }
    }

    /// Because we make checks when creating JqTeamplate to prevent the creation
    /// of const JqTemplate we can safely return always false
    pub fn is_const(&self) -> bool {
        false
    }
}

impl std::fmt::Debug for JqTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTemplate")
            .field("representation", &self.representation)
            .finish()
    }
}

impl Display for JqTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("[JqTemplate]({})", self.representation).fmt(f)
    }
}

impl std::cmp::PartialEq for JqTransform {
    fn eq(&self, other: &Self) -> bool {
        self.representation.eq(&other.representation)
    }
}

impl std::hash::Hash for JqTransform {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JqTemplateError {
    #[error("{0}")]
    Reason(String),
    #[error("JQ Load Errors: {0:?}")]
    JqLoadError(Vec<String>),
    #[error("JQ Compile Errors: {0:?}")]
    JqCompileError(Vec<String>),
    #[error("JQ Transform can be replaced with a Mustache")]
    JqIsMustache,
    #[error("JQ Transform can be replaced with a Literal")]
    JqIstConst,
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use serde_json::json;

    use super::*;

    #[test]
    fn test_is_mustache_simple_property() {
        let term = JqTransform::parse_template(".fruit");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for simple property access"
        );
    }

    #[test]
    fn test_is_mustache_nested_property() {
        let term = JqTransform::parse_template(".fruit.name");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for nested property access"
        );
    }

    #[test]
    fn test_is_mustache_optional() {
        let term = JqTransform::parse_template(".fruit.name?");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for optional operator"
        );
    }

    #[test]
    fn test_is_mustache_array_index() {
        let term = JqTransform::parse_template(".fruits[1]");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_mustache_pipe_operator() {
        let term = JqTransform::parse_template(".fruits[] | .name");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_mustache_filter() {
        let term = JqTransform::parse_template(".fruits[] | select(.price > 1)");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_mustache_true_value() {
        let term = JqTransform::parse_template("true");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for const true value"
        );
    }

    #[test]
    fn test_is_mustache_false_value() {
        let term = JqTransform::parse_template("false");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for const false value"
        );
    }

    #[test]
    fn test_is_mustache_number_value() {
        let term = JqTransform::parse_template("1");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for number value"
        );
    }

    #[test]
    fn test_is_mustache_str_value() {
        let term = JqTransform::parse_template("\"foobar\"");
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for string value"
        );
    }

    #[test]
    fn test_is_mustache_str_interpolate_value() {
        let term = JqTransform::parse_template("\"Hello, \\(.name)!\"");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for string interpolated value"
        );
    }

    #[test]
    fn test_is_mustache_function_call() {
        let term = JqTransform::parse_template("map(.price)");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for function call"
        );
    }

    #[test]
    fn test_is_mustache_concat() {
        let term = JqTransform::parse_template(".data.meat + .data.eggs");
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for concatenation"
        );
    }

    #[test]
    fn test_is_const_simple_property() {
        let term = JqTransform::parse_template(".fruit");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for simple property access"
        );
    }

    #[test]
    fn test_is_const_nested_property() {
        let term = JqTransform::parse_template(".fruit.name");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for nested property access"
        );
    }

    #[test]
    fn test_is_const_array_index() {
        let term = JqTransform::parse_template(".fruits[1]");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_const_pipe_operator() {
        let term = JqTransform::parse_template(".fruits[] | .name");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_const_filter() {
        let term = JqTransform::parse_template(".fruits[] | select(.price > 1)");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_const_true_value() {
        let term = JqTransform::parse_template("true");
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for const true value"
        );
    }

    #[test]
    fn test_is_const_false_value() {
        let term = JqTransform::parse_template("false");
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for const false value"
        );
    }

    #[test]
    fn test_is_const_number_value() {
        let term = JqTransform::parse_template("1");
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for number value"
        );
    }

    #[test]
    fn test_is_const_str_value() {
        let term = JqTransform::parse_template("\"foobar\"");
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for string value"
        );
    }

    #[test]
    fn test_is_const_str_interpolate_value() {
        let term = JqTransform::parse_template("\"Hello, \\(.name)!\"");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for string interpolated value"
        );
    }

    #[test]
    fn test_is_const_function_call() {
        let term = JqTransform::parse_template("map(.price)");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for function call"
        );
    }

    #[test]
    fn test_is_const_concat() {
        let term = JqTransform::parse_template(".data.meat + .data.eggs");
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for concatenation"
        );
    }

    #[test]
    fn test_render_value_no_results() {
        let template_str = ".[] | select(.non_existent)";
        let jq_template = JqTransform::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(PathValueEnum::Val(Val::from(input_json)));
        assert_eq!(
            result,
            async_graphql_value::ConstValue::Null,
            "Expected Null for no results"
        );
    }

    #[test]
    fn test_render_value_single_result() {
        let template_str = ".[0]";
        let jq_template = JqTransform::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(PathValueEnum::Val(Val::from(input_json)));
        assert_eq!(
            result,
            async_graphql_value::ConstValue::from_json(json!({"foo": 1})).unwrap(),
            "Expected single result"
        );
    }

    #[test]
    fn test_render_value_multiple_results() {
        let template_str = ".[] | .foo";
        let jq_template = JqTransform::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(PathValueEnum::Val(Val::from(input_json)));
        let expected = async_graphql_value::ConstValue::array(vec![
            async_graphql_value::ConstValue::from_json(json!(1)).unwrap(),
            async_graphql_value::ConstValue::from_json(json!(2)).unwrap(),
        ]);
        assert_eq!(result, expected, "Expected array of results");
    }

    #[test]
    fn test_debug() {
        let jq_template: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        let debug_string = format!("{:?}", jq_template);
        assert_eq!(debug_string, "JqTemplate { representation: \"test\" }");
    }

    #[test]
    fn test_display() {
        let jq_template: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        let display_string = format!("{}", jq_template);
        assert_eq!(display_string, "[JqTemplate](test)");
    }

    #[test]
    fn test_partial_eq() {
        let jq_template1: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        let jq_template2: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        assert_eq!(jq_template1, jq_template2);
    }

    #[test]
    fn test_hash() {
        let jq_template1: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        let jq_template2: JqTransform =
            JqTransform { template_id: 0, representation: "test".to_string() };
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        jq_template1.hash(&mut hasher1);
        jq_template2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
