use std::fmt::Display;

use jaq_core::load::parse::Term;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, Filter, Native, RcIter, ValR};

use super::PathValueEnum;
use crate::core::json::JsonLike;
/// Used to represent a JQ template. Currently used only on @expr directive.
#[derive(Clone)]
pub struct JqTransform {
    template: String,
    /// The IR representation, used for debug purposes
    representation: String,
}

impl JqTransform {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &str) -> Result<Self, JqRuntimeError> {
        let template = template.replace("\\\"", "\"");

        // the term is used because it can be easily serialized, deserialized and hashed
        let term = Self::parse_template(&template).map_err(JqRuntimeError::JqTemplateErrors)?;

        // calculate if the expression can be replaced with mustache
        let is_mustache = Self::recursive_is_mustache(&term);
        if is_mustache {
            return Err(JqRuntimeError::JqIsMustache);
        }

        // calculate if the expression returns always a constant value
        let is_const = Self::calculate_is_const(&term);
        if is_const {
            return Err(JqRuntimeError::JqIstConst);
        }

        Self::compile_template(&template)?;

        Ok(Self {
            template: template.to_string(),
            representation: format!("{:?}", term),
        })
    }

    /// Used to get the template string
    pub fn template(&self) -> &str {
        &self.template
    }

    fn compile_template(
        template: &str,
    ) -> Result<Filter<Native<PathValueEnum<'_>>>, JqRuntimeError> {
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
            JqRuntimeError::JqTemplateErrors(
                errs.into_iter()
                    .map(|err| JqTemplateError::JqLoadError(format!("{:?}", err)))
                    .collect::<Vec<_>>(),
            )
        })?;

        // the AST of the operation, used to transform the data
        let filter = Compiler::<_, Native<PathValueEnum>>::default()
            .with_funs(jaq_std::funs())
            .compile(modules)
            .map_err(|errs| {
                JqRuntimeError::JqTemplateErrors(
                    errs.into_iter()
                        .map(|err| JqTemplateError::JqCompileError(format!("{:?}", err)))
                        .collect::<Vec<_>>(),
                )
            })?;

        Ok(filter)
    }

    /// Used to execute the transformation of the JqTemplate
    pub fn run<'input>(
        &'input self,
        data: PathValueEnum<'input>,
    ) -> Vec<ValR<PathValueEnum<'input>>> {
        let inputs = RcIter::new(core::iter::empty());
        let ctx = Ctx::new([], &inputs);

        let filter: Filter<Native<PathValueEnum<'input>>> =
            Self::compile_template(&self.template).unwrap();

        filter.run((ctx, data)).collect::<Vec<_>>()
    }

    /// Used to calculate the result and return it as json
    pub fn render_value(
        &self,
        value: PathValueEnum<'_>,
    ) -> Result<async_graphql_value::ConstValue, JqRuntimeError> {
        let (errors, result): (Vec<_>, Vec<_>) = self
            .run(value)
            .into_iter()
            .map(|v| match v {
                Ok(v) => Ok(std::convert::Into::into(v)),
                Err(err) => Err(err),
            })
            .map(|v| match v {
                Ok(v) => async_graphql_value::ConstValue::from_json(v)
                    .map_err(|err| JqTemplateError::JsonParseError(err.to_string())),
                Err(err) => Err(JqTemplateError::JqRuntimeError(err.to_string())),
            })
            .partition(Result::is_err);

        let errors: Vec<JqTemplateError> = errors
            .into_iter()
            .filter_map(|e| match e {
                Ok(_) => None,
                Err(err) => Some(err),
            })
            .collect();
        if !errors.is_empty() {
            return Err(JqRuntimeError::JqTemplateErrors(errors));
        }

        let result: Vec<_> = result
            .into_iter()
            .filter_map(|v| match v {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .collect();

        // convert results to graphql value
        let res_len = result.len();
        if res_len == 0 {
            Ok(async_graphql_value::ConstValue::Null)
        } else if res_len == 1 {
            Ok(result.into_iter().next().unwrap())
        } else {
            Ok(async_graphql_value::ConstValue::array(result))
        }
    }

    /// Used to parse the template string and return the IR representation
    fn parse_template(template: &str) -> Result<Term<&str>, Vec<JqTemplateError>> {
        let lexer = jaq_core::load::Lexer::new(template);
        let lex = lexer.lex().map_err(|err| {
            err.into_iter()
                .map(|err| JqTemplateError::JqLexError(format!("{:?}", err)))
                .collect::<Vec<_>>()
        })?;
        let mut parser = jaq_core::load::parse::Parser::new(&lex);
        parser
            .term()
            .map_err(|err| vec![JqTemplateError::JqParseError(format!("{:?}", err))])
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

#[derive(Clone, Debug, thiserror::Error)]
pub enum JqTemplateError {
    #[error("[JQ Load Errors] {0:?}")]
    JqLoadError(String),
    #[error("[JQ Compile Error] {0:?}")]
    JqCompileError(String),
    #[error("[JQ Parse Error] {0:?}")]
    JqParseError(String),
    #[error("[JQ Lex Error] {0:?}")]
    JqLexError(String),
    #[error("[JQ Runtime Error] {0:?}")]
    JqRuntimeError(String),
    #[error("[JQ Json Parse Error] {0:?}")]
    JsonParseError(String),
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum JqRuntimeError {
    #[error("{0:?}")]
    JqTemplateErrors(Vec<JqTemplateError>),
    #[error("{0:?}")]
    JqRuntimeErrors(Vec<Self>),
    #[error("JQ Transform can be replaced with a Mustache.")]
    JqIsMustache,
    #[error("JQ Transform can be replaced with a Literal.")]
    JqIstConst,
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use jaq_json::Val;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_is_mustache_simple_property() {
        let term = JqTransform::parse_template(".fruit").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for simple property access"
        );
    }

    #[test]
    fn test_is_mustache_nested_property() {
        let term = JqTransform::parse_template(".fruit.name").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for nested property access"
        );
    }

    #[test]
    fn test_is_mustache_optional() {
        let term = JqTransform::parse_template(".fruit.name?").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for optional operator"
        );
    }

    #[test]
    fn test_is_mustache_array_index() {
        let term = JqTransform::parse_template(".fruits[1]").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_mustache_pipe_operator() {
        let term = JqTransform::parse_template(".fruits[] | .name").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_mustache_filter() {
        let term = JqTransform::parse_template(".fruits[] | select(.price > 1)").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_mustache_true_value() {
        let term = JqTransform::parse_template("true").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for const true value"
        );
    }

    #[test]
    fn test_is_mustache_false_value() {
        let term = JqTransform::parse_template("false").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for const false value"
        );
    }

    #[test]
    fn test_is_mustache_number_value() {
        let term = JqTransform::parse_template("1").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for number value"
        );
    }

    #[test]
    fn test_is_mustache_str_value() {
        let term = JqTransform::parse_template("\"foobar\"").unwrap();
        assert!(
            JqTransform::recursive_is_mustache(&term),
            "Should return true for string value"
        );
    }

    #[test]
    fn test_is_mustache_str_interpolate_value() {
        let term = JqTransform::parse_template("\"Hello, \\(.name)!\"").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for string interpolated value"
        );
    }

    #[test]
    fn test_is_mustache_function_call() {
        let term = JqTransform::parse_template("map(.price)").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for function call"
        );
    }

    #[test]
    fn test_is_mustache_concat() {
        let term = JqTransform::parse_template(".data.meat + .data.eggs").unwrap();
        assert!(
            !JqTransform::recursive_is_mustache(&term),
            "Should return false for concatenation"
        );
    }

    #[test]
    fn test_is_const_simple_property() {
        let term = JqTransform::parse_template(".fruit").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for simple property access"
        );
    }

    #[test]
    fn test_is_const_nested_property() {
        let term = JqTransform::parse_template(".fruit.name").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for nested property access"
        );
    }

    #[test]
    fn test_is_const_array_index() {
        let term = JqTransform::parse_template(".fruits[1]").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_const_pipe_operator() {
        let term = JqTransform::parse_template(".fruits[] | .name").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_const_filter() {
        let term = JqTransform::parse_template(".fruits[] | select(.price > 1)").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_const_true_value() {
        let term = JqTransform::parse_template("true").unwrap();
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for const true value"
        );
    }

    #[test]
    fn test_is_const_false_value() {
        let term = JqTransform::parse_template("false").unwrap();
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for const false value"
        );
    }

    #[test]
    fn test_is_const_number_value() {
        let term = JqTransform::parse_template("1").unwrap();
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for number value"
        );
    }

    #[test]
    fn test_is_const_str_value() {
        let term = JqTransform::parse_template("\"foobar\"").unwrap();
        assert!(
            JqTransform::calculate_is_const(&term),
            "Should return true for string value"
        );
    }

    #[test]
    fn test_is_const_str_interpolate_value() {
        let term = JqTransform::parse_template("\"Hello, \\(.name)!\"").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for string interpolated value"
        );
    }

    #[test]
    fn test_is_const_function_call() {
        let term = JqTransform::parse_template("map(.price)").unwrap();
        assert!(
            !JqTransform::calculate_is_const(&term),
            "Should return false for function call"
        );
    }

    #[test]
    fn test_is_const_concat() {
        let term = JqTransform::parse_template(".data.meat + .data.eggs").unwrap();
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
        let result = jq_template
            .render_value(PathValueEnum::Val(Val::from(input_json)))
            .unwrap();
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
        let result = jq_template
            .render_value(PathValueEnum::Val(Val::from(input_json)))
            .unwrap();
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
        let result = jq_template
            .render_value(PathValueEnum::Val(Val::from(input_json)))
            .unwrap();
        let expected = async_graphql_value::ConstValue::array(vec![
            async_graphql_value::ConstValue::from_json(json!(1)).unwrap(),
            async_graphql_value::ConstValue::from_json(json!(2)).unwrap(),
        ]);
        assert_eq!(result, expected, "Expected array of results");
    }

    #[test]
    fn test_debug() {
        let jq_template: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        let debug_string = format!("{:?}", jq_template);
        assert_eq!(debug_string, "JqTemplate { representation: \"test\" }");
    }

    #[test]
    fn test_display() {
        let jq_template: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        let display_string = format!("{}", jq_template);
        assert_eq!(display_string, "[JqTemplate](test)");
    }

    #[test]
    fn test_partial_eq() {
        let jq_template1: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        let jq_template2: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        assert_eq!(jq_template1, jq_template2);
    }

    #[test]
    fn test_hash() {
        let jq_template1: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        let jq_template2: JqTransform =
            JqTransform { template: "".to_string(), representation: "test".to_string() };
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        jq_template1.hash(&mut hasher1);
        jq_template2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
