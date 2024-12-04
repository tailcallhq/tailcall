use std::fmt::Display;
use std::sync::Arc;

use jaq_core::load::parse::Term;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, Filter, Native, RcIter, ValR};
use jaq_json::Val;

#[derive(Clone)]
pub struct JqTransformer {
    filter: Arc<Filter<Native<Val>>>,
    representation: String,
    is_const: bool,
}

impl JqTransformer {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &str) -> Result<Self, JqTemplateError> {
        // the term is used because it can be easily serialized, deserialized and hashed
        let term = Self::parse_template(template);
        let is_const = Self::calculate_is_const(&term);

        // the template is used to be parsed in to the IR AST
        let template = File { code: template, path: () };
        // defs is used to extend the syntax with custom definitions of functions, like
        // 'toString'
        let defs = vec![];
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

    /// Used to execute the transformation of the JQTemplate
    pub fn run<'a, T: std::iter::Iterator<Item = std::result::Result<Val, std::string::String>>>(
        &'a self,
        inputs: &'a RcIter<T>,
        data: Val,
    ) -> impl Iterator<Item = ValR<Val>> + 'a {
        let ctx = Ctx::new([], inputs);
        self.filter.run((ctx, data))
    }

    pub fn render(&self, value: serde_json::Value) -> String {
        self.render_helper(Val::from(value))
    }

    pub fn render_graphql(&self, value: &async_graphql_value::ConstValue) -> String {
        let Ok(value) = value.clone().into_json() else {
            return String::default();
        };

        self.render_helper(Val::from(value))
    }

    fn render_helper(&self, value: Val) -> String {
        // the hardcoded inputs for the AST
        let inputs = RcIter::new(core::iter::empty());
        let res = self.run(&inputs, value);
        res.filter_map(|v| if let Ok(v) = v { Some(v) } else { None })
            .fold(String::new(), |acc, cur| {
                let cur_string = cur.to_string();
                acc + &cur_string
            })
    }

    /// Used to determine if the expression can be supported with current
    /// Mustache implementation
    pub fn is_select_operation(template: &str) -> bool {
        let term = Self::parse_template(template);
        Self::recursive_is_select_operation(term)
    }

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

impl Default for JqTransformer {
    fn default() -> Self {
        Self {
            filter: Default::default(),
            representation: String::default(),
            is_const: true,
        }
    }
}

impl std::fmt::Debug for JqTransformer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTransformer")
            .field("representation", &self.representation)
            .finish()
    }
}

impl Display for JqTransformer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!(
            "[JqTransformer](is_const={})({})",
            self.is_const, self.representation
        )
        .fmt(f)
    }
}

impl std::cmp::PartialEq for JqTransformer {
    fn eq(&self, other: &Self) -> bool {
        // TODO: sorry for the quick hack
        self.representation.eq(&other.representation)
    }
}

impl std::hash::Hash for JqTransformer {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_select_operation_simple_property() {
        let template = ".fruit";
        assert!(
            JqTransformer::is_select_operation(template),
            "Should return true for simple property access"
        );
    }

    #[test]
    fn test_is_select_operation_nested_property() {
        let template = ".fruit.name";
        assert!(
            JqTransformer::is_select_operation(template),
            "Should return true for nested property access"
        );
    }

    #[test]
    fn test_is_select_operation_array_index() {
        let template = ".fruits[1]";
        assert!(
            !JqTransformer::is_select_operation(template),
            "Should return false for array index access"
        );
    }

    #[test]
    fn test_is_select_operation_pipe_operator() {
        let template = ".fruits[] | .name";
        assert!(
            !JqTransformer::is_select_operation(template),
            "Should return false for pipe operator usage"
        );
    }

    #[test]
    fn test_is_select_operation_filter() {
        let template = ".fruits[] | select(.price > 1)";
        assert!(
            !JqTransformer::is_select_operation(template),
            "Should return false for select filter usage"
        );
    }

    #[test]
    fn test_is_select_operation_function_call() {
        let template = "map(.price)";
        assert!(
            !JqTransformer::is_select_operation(template),
            "Should return false for function call"
        );
    }
}
