use std::fmt::Display;
use std::sync::Arc;

use jaq_core::load::parse::Term;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, Filter, Native, RcIter, ValR};
use jaq_json::Val;
use regex::Regex;

use crate::core::json::JsonLike;

/// Used to represent a JQ template. Currently used only on @expr directive.
#[derive(Clone)]
pub struct JqTemplate {
    /// The compiled filter
    filter: Arc<Filter<Native<Val>>>,
    /// The IR representation, used for debug purposes
    representation: String,
    /// If the transformer returns a constant value
    is_const: bool,
}

impl JqTemplate {
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
        let filter = Compiler::<_, Native<Val>>::default()
            .with_funs(jaq_std::funs())
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
    pub fn run<'a, T: std::iter::Iterator<Item = std::result::Result<Val, std::string::String>>>(
        &'a self,
        inputs: &'a RcIter<T>,
        data: Val,
    ) -> impl Iterator<Item = ValR<Val>> + 'a {
        let ctx = Ctx::new([], inputs);
        self.filter.run((ctx, data))
    }

    /// Used to calculate the result and return it as json
    pub fn render_value(&self, value: serde_json::Value) -> async_graphql_value::ConstValue {
        let inputs = RcIter::new(core::iter::empty());
        let res = self.run(&inputs, Val::from(value));
        let res: Vec<async_graphql_value::ConstValue> = res
            .into_iter()
            // TODO: handle error correct, now we ignore it
            .filter_map(|v| if let Ok(v) = v { Some(v) } else { None })
            .map(serde_json::Value::from)
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

impl Default for JqTemplate {
    fn default() -> Self {
        Self {
            filter: Default::default(),
            representation: String::default(),
            is_const: true,
        }
    }
}

impl std::fmt::Debug for JqTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTemplate")
            .field("representation", &self.representation)
            .finish()
    }
}

impl Display for JqTemplate {
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

    #[test]
    fn test_render_value_no_results() {
        let template_str = ".[] | select(.non_existent)";
        let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(input_json);
        assert_eq!(
            result,
            async_graphql_value::ConstValue::Null,
            "Expected Null for no results"
        );
    }

    #[test]
    fn test_render_value_single_result() {
        let template_str = ".[0]";
        let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(input_json);
        assert_eq!(
            result,
            async_graphql_value::ConstValue::from_json(json!({"foo": 1})).unwrap(),
            "Expected single result"
        );
    }

    #[test]
    fn test_render_value_multiple_results() {
        let template_str = ".[] | .foo";
        let jq_template = JqTemplate::try_new(template_str).expect("Failed to create JqTemplate");
        let input_json = json!([{"foo": 1}, {"foo": 2}]);
        let result = jq_template.render_value(input_json);
        let expected = async_graphql_value::ConstValue::array(vec![
            async_graphql_value::ConstValue::from_json(json!(1)).unwrap(),
            async_graphql_value::ConstValue::from_json(json!(2)).unwrap(),
        ]);
        assert_eq!(result, expected, "Expected array of results");
    }

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
        let jq_template = JqTemplate::default();
        assert_eq!(jq_template.representation, "");
        assert!(jq_template.is_const);
        // Assuming `filter` has a sensible default implementation
    }

    #[test]
    fn test_debug() {
        let jq_template = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let debug_string = format!("{:?}", jq_template);
        assert_eq!(debug_string, "JqTemplate { representation: \"test\" }");
    }

    #[test]
    fn test_display() {
        let jq_template = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let display_string = format!("{}", jq_template);
        assert_eq!(display_string, "[JqTemplate](is_const=false)(test)");
    }

    #[test]
    fn test_partial_eq() {
        let jq_template1 = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let jq_template2 = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: true, // Different `is_const` value should not affect equality
        };
        assert_eq!(jq_template1, jq_template2);
    }

    #[test]
    fn test_hash() {
        let jq_template1 = JqTemplate {
            filter: Arc::new(Filter::default()),
            representation: "test".to_string(),
            is_const: false,
        };
        let jq_template2 = JqTemplate {
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
