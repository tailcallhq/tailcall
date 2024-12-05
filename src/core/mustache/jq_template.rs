use std::fmt::Display;
use std::sync::Arc;

use jaq_core::load::parse::Term;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, Error, Filter, Native, RcIter, ValR, ValT};
use jaq_json::Val;
use regex::Regex;

use crate::core::ir::{EvalContext, ResolverContextLike};
use crate::core::json::JsonLike;
use crate::core::path::{PathString, ValueString};

/// Used to represent a JQ template. Currently used only on @expr directive.
#[derive(Clone)]
pub struct JqTemplate {
    /// The compiled filter
    filter: Arc<Filter<Native<PathValueEnum>>>,
    /// The IR representation, used for debug purposes
    representation: String,
    /// If the transformer returns a constant value
    is_const: bool,
}

#[derive(Clone)]
pub enum PathValueEnum {
    PathValue(Arc<dyn PathJqValue>),
    Val(Val)
}

pub trait PathJqValue {
    fn get_value<'a>(&'a self, path: &str) -> Option<ValueString<'a>>;
}

impl<Ctx: ResolverContextLike> PathJqValue for EvalContext<'_, Ctx> {
    fn get_value<'a>(&'a self, path: &str) -> Option<ValueString<'a>> {
        todo!()
    }
}

impl PathJqValue for serde_json::Value {
    fn get_value<'a>(&'a self, path: &str) -> Option<ValueString<'a>> {
        todo!()
    }
}
pub trait PathJqValueString: PathString + PathJqValue {}

impl<Ctx: ResolverContextLike> PathJqValueString for EvalContext<'_, Ctx> {}

impl PathJqValueString for serde_json::Value {}


impl ValT for PathValueEnum {
    fn from_num(n: &str) -> ValR<Self> {
        match Val::from_num(n) {
            Ok(val) => ValR::Ok(Self::Val(val)),
            Err(err) => {
                let val = err.into_val();
                Err(Error::new(Self::Val(val)))
            },
        }
    }

    fn from_map<I: IntoIterator<Item = (Self, Self)>>(iter: I) -> ValR<Self> {
        let result: Result<Vec<(Val, Val)>, String> = iter.into_iter().map(|(k, v)| {
            match (k, v) {
                (PathValueEnum::Val(key), PathValueEnum::Val(value)) => Ok((key, value)),
                _ => Err("Invalid key or value type for map".into())
            }
        }).collect();

        match result {
            Ok(pairs) => {
                match Val::from_map(pairs.into_iter()) {
                    Ok(val) => ValR::Ok(PathValueEnum::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    },
                }
            },
            Err(e) => Err(Error::new(Self::Val(Val::from(e))))
        }
    }

    fn values(self) -> Box<dyn Iterator<Item = ValR<Self>>> {
        match self {
            PathValueEnum::PathValue(_context) => {
                // Create a new error message each time to avoid lifetime issues
                let error_message = "Cannot iterate context.".to_string();
                let error_val = Val::from(error_message);
                let error = Error::new(Self::Val(error_val));
                Box::new(std::iter::once(Err(error)))
            },
            PathValueEnum::Val(val) => {
                Box::new(std::iter::once(Ok(PathValueEnum::Val(val))))
            }
        }
    }

    fn index(self, index: &Self) -> ValR<Self> {
        let PathValueEnum::Val(index) = index else {
            return ValR::Err(Error::new(Self::Val(Val::from(format!("Could not convert index `{}` val.", index)))));
        };

        match self {
            PathValueEnum::PathValue(pv) => {
                let Some(index) = index.as_str() else {
                    return ValR::Err(Error::new(Self::Val(Val::from(format!("Could not convert index `{}` to string.", index)))));
                };

                let Some(v) = pv.get_value(index) else {
                    return ValR::Err(Error::new(Self::Val(Val::from(format!("Could not find key `{}` in context.", index)))));
                };

                match v {
                    crate::core::path::ValueString::Value(cow) => {
                        let cv = cow.as_ref().clone();
                        match cv.into_json() {
                            Ok(js) => Ok(Self::Val(Val::from(js))),
                            Err(err) => ValR::Err(Error::new(Self::Val(Val::from(format!("Could not convert value to json: {:?}", err))))),
                        }
                    },
                    crate::core::path::ValueString::String(cow) => {
                        let v = cow.to_string();
                        Ok(Self::Val(Val::from(v)))
                    },
                }
            },
            PathValueEnum::Val(val) => {
                match val.index(index) {
                    Ok(val) => ValR::Ok(Self::Val(val)),
                    Err(err) => {
                        let val = err.into_val();
                        Err(Error::new(Self::Val(val)))
                    },
                }
            },
        }
    }

    fn range(self, range: jaq_core::val::Range<&Self>) -> ValR<Self> {
        todo!()
    }

    fn map_values<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn map_index<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        index: &Self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn map_range<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        range: jaq_core::val::Range<&Self>,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn as_bool(&self) -> bool {
        todo!()
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            PathValueEnum::PathValue(_) => None,
            PathValueEnum::Val(val) => val.as_str(),
        }
    }
}

impl FromIterator<PathValueEnum> for PathValueEnum {
    fn from_iter<I: IntoIterator<Item = PathValueEnum>>(iter: I) -> Self {
        todo!()
    }
}

impl std::ops::Add for PathValueEnum {
    type Output = ValR<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Sub for PathValueEnum {
    type Output = ValR<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Mul for PathValueEnum {
    type Output = ValR<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Div for PathValueEnum {
    type Output = ValR<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Rem for PathValueEnum {
    type Output = ValR<Self>;

    fn rem(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Neg for PathValueEnum {
    type Output = ValR<Self>;

    fn neg(self) -> Self::Output {
        todo!()
    }
}

impl Display for PathValueEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
            // PathValueEnum::PathValue(_) => "[PathValue]".to_string().fmt(f),
            // PathValueEnum::Val(val) => val.fmt(f),
        // }
        todo!()
    }
}

impl std::fmt::Debug for PathValueEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     Self::PathValue(arg0) => f.debug_tuple("PathValue").field(arg0).finish(),
        //     Self::Val(arg0) => f.debug_tuple("Val").field(arg0).finish(),
        // }
        todo!()
    }
}

impl PartialEq for PathValueEnum {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for PathValueEnum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl Into<serde_json::Value> for PathValueEnum {
    fn into(self) -> serde_json::Value {
        match self {
            PathValueEnum::PathValue(_) => todo!(),
            PathValueEnum::Val(val) => serde_json::Value::from(val),
        }
    }
}

impl From<String> for PathValueEnum {
    fn from(value: String) -> Self {
        todo!()
    }
}

impl From<isize> for PathValueEnum {
    fn from(value: isize) -> Self {
        todo!()
    }
}

impl From<bool> for PathValueEnum {
    fn from(value: bool) -> Self {
        todo!()
    }
}

impl  JqTemplate {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &str) -> Result<Self, JqTemplateError> {
        let template = transform_to_jq(template);

        // the term is used because it can be easily serialized, deserialized and hashed
        let term = Self::parse_template(&template);
        // calculate if the expression returns always a constant value
        let is_const = Self::calculate_is_const(&term);

        // the template is used to be parsed in to the IR AST
        let template = File { code: template.as_str(), path: () };
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
            // .with_funs(jaq_std::funs())
            .compile(modules)
            .map_err(|errs| {
                JqTemplateError::JqCompileError(
                    errs.into_iter().map(|e| format!("{:?}", e.1)).collect(),
                )
            })?;

        Ok(Self {
            filter: Arc::new(filter),
            representation: format!("{:?}", term),
            is_const,
        })
    }

    /// Used to execute the transformation of the JqTemplate
    pub fn run<'a, Y: std::iter::Iterator<Item = std::result::Result<PathValueEnum, std::string::String>>>(
        &'a self,
        inputs: &'a RcIter<Y>,
        data: PathValueEnum,
    ) -> impl Iterator<Item = ValR<PathValueEnum>> + 'a {
        let ctx = Ctx::new([], inputs);
        self.filter.run((ctx, data))
    }

    /// Used to calculate the result and return it as json
    pub fn render_value(&self, value: PathValueEnum) -> async_graphql_value::ConstValue {
        let inputs = RcIter::new(core::iter::empty());
        let res = self.run(&inputs, value);
        let res: Vec<async_graphql_value::ConstValue> = res
            .into_iter()
            // TODO: handle error correct, now we ignore it
            .filter_map(|v| if let Ok(v) = v { Some(v) } else { None })
            .map(|v| std::convert::Into::into(v))
            .map(async_graphql_value::ConstValue::from_json)
            // TODO: handle error correct, now we ignore it
            .filter_map(|v| if let Ok(v) = v { Some(v) } else { None })
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

    /// Used to determine if the expression can be supported with current
    /// Mustache implementation
    pub fn is_select_operation(template: &str) -> bool {
        let term = Self::parse_template(template);
        Self::recursive_is_select_operation(term)
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
    fn recursive_is_select_operation(term: Term<&str>) -> bool {
        match term {
            Term::Id => true,
            Term::Recurse => false,
            Term::Num(_) => false,
            Term::Str(formater, _) => formater.is_none(),
            Term::Arr(_) => false,
            Term::Obj(_) => false,
            Term::Neg(_) => false,
            Term::Pipe(local_term_1, pattern, local_term_2) => {
                if pattern.is_some() {
                    false
                } else {
                    Self::recursive_is_select_operation(*local_term_1)
                        && Self::recursive_is_select_operation(*local_term_2)
                }
            }
            Term::BinOp(_, _, _) => false,
            Term::Label(_, _) => false,
            Term::Break(_) => false,
            Term::Fold(_, _, _, _) => false,
            Term::TryCatch(_, _) => false,
            Term::IfThenElse(_, _) => false,
            Term::Def(_, _) => false,
            Term::Call(_, _) => false,
            Term::Var(_) => false,
            Term::Path(local_term, path) => {
                Self::recursive_is_select_operation(*local_term)
                    && Self::is_path_select_operation(path)
            }
        }
    }

    /// Used to check if the path indicates a select operation or modify
    fn is_path_select_operation(path: jaq_core::path::Path<Term<&str>>) -> bool {
        path.0.into_iter().all(|part| match part {
            (jaq_core::path::Part::Index(idx), jaq_core::path::Opt::Optional) => {
                Self::recursive_is_select_operation(idx)
            }
            (jaq_core::path::Part::Index(idx), jaq_core::path::Opt::Essential) => {
                Self::recursive_is_select_operation(idx)
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
            Term::Num(_) => true,
            Term::Str(formater, _) => formater.is_none(),
            Term::Arr(_) => false,
            Term::Obj(_) => false,
            Term::Neg(_) => false,
            Term::Pipe(local_term_1, pattern, local_term_2) => {
                if pattern.is_some() {
                    false
                } else {
                    Self::calculate_is_const(local_term_1) && Self::calculate_is_const(local_term_2)
                }
            }
            Term::BinOp(_, _, _) => false,
            Term::Label(_, _) => false,
            Term::Break(_) => false,
            Term::Fold(_, _, _, _) => false,
            Term::TryCatch(_, _) => false,
            Term::IfThenElse(_, _) => false,
            Term::Def(_, _) => false,
            Term::Call(_, _) => false,
            Term::Var(_) => false,
            Term::Path(_, _) => false,
        }
    }

    /// Used to determine if the transformer is a static value
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

impl  Default for JqTemplate {
    fn default() -> Self {
        Self {
            filter: Default::default(),
            representation: String::default(),
            is_const: true,
        }
    }
}

impl  std::fmt::Debug for JqTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTemplate")
            .field("representation", &self.representation)
            .finish()
    }
}

impl  Display for JqTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!(
            "[JqTemplate](is_const={})({})",
            self.is_const, self.representation
        )
        .fmt(f)
    }
}

impl std::cmp::PartialEq for JqTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.representation.eq(&other.representation)
    }
}

impl std::hash::Hash for JqTemplate {
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
}

/// Used to convert mustache to jq
fn transform_to_jq(input: &str) -> String {
    let re = Regex::new(r"\{\{\.([^}]*)\}\}").unwrap();
    let mut result = String::new();
    let mut last_end = 0;
    let captures: Vec<_> = re.captures_iter(input).collect();

    // when we do not have any mustache templates return the string
    if captures.is_empty() {
        return input.to_string();
    }

    for cap in captures {
        let match_ = cap.get(0).unwrap();
        let var_name = cap.get(1).unwrap().as_str();

        // Append the text before the match, then the transformed variable
        if last_end != match_.start() {
            if !result.is_empty() {
                result.push_str(" + ");
            }
            result.push_str(&format!("\"{}\"", &input[last_end..match_.start()]));
        }

        if !result.is_empty() {
            result.push_str(" + ");
        }
        result.push_str(&format!(".{}", var_name));

        last_end = match_.end();
    }

    // Append any remaining text after the last match
    if last_end < input.len() {
        if !result.is_empty() {
            result.push_str(" + ");
        }
        result.push_str(&format!("\"{}\"", &input[last_end..]));
    }

    // If no transformations were made, return the original input
    if result.is_empty() {
        return input.to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use jaq_core::load::parse::{BinaryOp, Pattern, Term};
    use serde_json::json;

    use super::*;

    #[test]
    fn test_is_select_operation_simple_property() {
        let template = ".fruit";
        assert!(
            JqTemplate::is_select_operation(template),
            "Should return true for simple property access"
        );
    }

    #[test]
    fn test_is_select_operation_nested_property() {
        let template = ".fruit.name";
        assert!(
            JqTemplate::is_select_operation(template),
            "Should return true for nested property access"
        );
    }

    #[test]
    fn test_is_select_operation_array_index() {
        let template = ".fruits[1]";
        assert!(
            !JqTemplate::is_select_operation(template),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_select_operation_pipe_operator() {
        let template = ".fruits[] | .name";
        assert!(
            !JqTemplate::is_select_operation(template),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_select_operation_filter() {
        let template = ".fruits[] | select(.price > 1)";
        assert!(
            !JqTemplate::is_select_operation(template),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_select_operation_function_call() {
        let template = "map(.price)";
        assert!(
            !JqTemplate::is_select_operation(template),
            "Should return false for function call"
        );
    }

    // #[test]
    // fn test_render_value_no_results() {
    //     let template_str = ".[] | select(.non_existent)";
    //     let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
    //     let input_json = json!([{"foo": 1}, {"foo": 2}]);
    //     let result = jq_template.render_value(input_json);
    //     assert_eq!(
    //         result,
    //         async_graphql_value::ConstValue::Null,
    //         "Expected Null for no results"
    //     );
    // }

    // #[test]
    // fn test_render_value_single_result() {
    //     let template_str = ".[0]";
    //     let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
    //     let input_json = json!([{"foo": 1}, {"foo": 2}]);
    //     let result = jq_template.render_value(input_json);
    //     assert_eq!(
    //         result,
    //         async_graphql_value::ConstValue::from_json(json!({"foo": 1})).unwrap(),
    //         "Expected single result"
    //     );
    // }

    // #[test]
    // fn test_render_value_multiple_results() {
    //     let template_str = ".[] | .foo";
    //     let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
    //     let input_json = json!([{"foo": 1}, {"foo": 2}]);
    //     let result = jq_template.render_value(input_json);
    //     let expected = async_graphql_value::ConstValue::array(vec![
    //         async_graphql_value::ConstValue::from_json(json!(1)).unwrap(),
    //         async_graphql_value::ConstValue::from_json(json!(2)).unwrap(),
    //     ]);
    //     assert_eq!(result, expected, "Expected array of results");
    // }

    #[test]
    fn test_calculate_is_const() {
        // Test with a constant number
        let term_num = Term::Num("42");
        assert!(
            JqTemplate::calculate_is_const(&term_num),
            "Expected true for a constant number"
        );

        // Test with a string without formatter
        let term_str = Term::Str(None, vec![]);
        assert!(
            JqTemplate::calculate_is_const(&term_str),
            "Expected true for a simple string"
        );

        // Test with a string with formatter
        let term_str_fmt = Term::Str(Some("fmt"), vec![]);
        assert!(
            !JqTemplate::calculate_is_const(&term_str_fmt),
            "Expected false for a formatted string"
        );

        // Test with an identity operation
        let term_id = Term::Id;
        assert!(
            !JqTemplate::calculate_is_const(&term_id),
            "Expected false for an identity operation"
        );

        // Test with a recursive operation
        let term_recurse = Term::Recurse;
        assert!(
            !JqTemplate::calculate_is_const(&term_recurse),
            "Expected false for a recursive operation"
        );

        // Test with a binary operation
        let term_bin_op = Term::BinOp(
            Box::new(Term::Num("1")),
            BinaryOp::Math(jaq_core::ops::Math::Add),
            Box::new(Term::Num("2")),
        );
        assert!(
            !JqTemplate::calculate_is_const(&term_bin_op),
            "Expected false for a binary operation"
        );

        // Test with a pipe operation without pattern
        let term_pipe = Term::Pipe(Box::new(Term::Num("1")), None, Box::new(Term::Num("2")));
        assert!(
            JqTemplate::calculate_is_const(&term_pipe),
            "Expected true for a constant pipe operation"
        );

        // Test with a pipe operation with pattern
        let pattern = Pattern::Var("x");
        let term_pipe_with_pattern = Term::Pipe(
            Box::new(Term::Num("1")),
            Some(pattern),
            Box::new(Term::Num("2")),
        );
        assert!(
            !JqTemplate::calculate_is_const(&term_pipe_with_pattern),
            "Expected false for a pipe operation with pattern"
        );
    }

    #[test]
    fn test_recursive_is_select_operation() {
        // Test with simple identity operation
        let term_id = Term::Id;
        assert!(
            JqTemplate::recursive_is_select_operation(term_id),
            "Expected true for identity operation"
        );

        // Test with a number
        let term_num = Term::Num("42");
        assert!(
            !JqTemplate::recursive_is_select_operation(term_num),
            "Expected false for a number"
        );

        // Test with a string without formatter
        let term_str = Term::Str(None, vec![]);
        assert!(
            JqTemplate::recursive_is_select_operation(term_str),
            "Expected true for a simple string"
        );

        // Test with a string with formatter
        let term_str_fmt = Term::Str(Some("fmt"), vec![]);
        assert!(
            !JqTemplate::recursive_is_select_operation(term_str_fmt),
            "Expected false for a formatted string"
        );

        // Test with a recursive operation
        let term_recurse = Term::Recurse;
        assert!(
            !JqTemplate::recursive_is_select_operation(term_recurse),
            "Expected false for a recursive operation"
        );

        // Test with a binary operation
        let term_bin_op = Term::BinOp(
            Box::new(Term::Num("1")),
            BinaryOp::Math(jaq_core::ops::Math::Add),
            Box::new(Term::Num("2")),
        );
        assert!(
            !JqTemplate::recursive_is_select_operation(term_bin_op),
            "Expected false for a binary operation"
        );

        // Test with a pipe operation without pattern
        let term_pipe = Term::Pipe(Box::new(Term::Num("1")), None, Box::new(Term::Num("2")));
        assert!(
            !JqTemplate::recursive_is_select_operation(term_pipe),
            "Expected false for a constant pipe operation"
        );

        // Test with a pipe operation with pattern
        let pattern = Pattern::Var("x");
        let term_pipe_with_pattern = Term::Pipe(
            Box::new(Term::Num("1")),
            Some(pattern),
            Box::new(Term::Num("2")),
        );
        assert!(
            !JqTemplate::recursive_is_select_operation(term_pipe_with_pattern),
            "Expected false for a pipe operation with pattern"
        );
    }

    #[test]
    fn test_default() {
        let jq_template: JqTemplate = JqTemplate::default();
        assert_eq!(jq_template.representation, "");
        assert!(jq_template.is_const);
        // Assuming `filter` has a sensible default implementation
    }

    #[test]
    fn test_debug() {
        let jq_template: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let debug_string = format!("{:?}", jq_template);
        assert_eq!(debug_string, "JqTemplate { representation: \"test\" }");
    }

    #[test]
    fn test_display() {
        let jq_template: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let display_string = format!("{}", jq_template);
        assert_eq!(display_string, "[JqTemplate](is_const=false)(test)");
    }

    #[test]
    fn test_partial_eq() {
        let jq_template1: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let jq_template2: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: true, // Different `is_const` value should not affect equality
        };
        assert_eq!(jq_template1, jq_template2);
    }

    #[test]
    fn test_hash() {
        let jq_template1: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let jq_template2: JqTemplate = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        jq_template1.hash(&mut hasher1);
        jq_template2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_transform_to_jq() {
        assert_eq!(
            transform_to_jq("Hello world: {{.foo.buzz | split(\" \")}}"),
            "\"Hello world: \" + .foo.buzz | split(\" \")"
        );
        assert_eq!(
            transform_to_jq("Hello world: {{.foo.buzz | split(\" \")}} this is great"),
            "\"Hello world: \" + .foo.buzz | split(\" \") + \" this is great\""
        );
        assert_eq!(
            transform_to_jq("{{.foo.buzz | split(\" \")}} buzz"),
            ".foo.buzz | split(\" \") + \" buzz\""
        );
        assert_eq!(
            transform_to_jq("{{.foo.buzz | split(\" \")}} of type {{.bar}}"),
            ".foo.buzz | split(\" \") + \" of type \" + .bar"
        );
    }

    #[test]
    fn test_transform_to_jq_identity() {
        assert_eq!(transform_to_jq("Hello world"), "Hello world");
        assert_eq!(
            transform_to_jq(".foo.buzz | split(\" \")"),
            ".foo.buzz | split(\" \")"
        );
    }
}
