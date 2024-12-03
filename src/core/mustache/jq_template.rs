use std::iter::Empty;

use jaq_core::{
    load::{parse::Term, Arena, File, Loader},
    Compiler, Ctx, Filter, Native, RcIter, ValR,
};
use jaq_json::Val;

use crate::core::ir::{EvalContext, ResolverContextLike};

use super::Mustache;

#[derive(Debug)]
pub enum JqTemplate {
    Mustache(Mustache),
    JqTemplate(JqTransformer)
}

impl JqTemplate {
    pub fn render(&self, value: &serde_json::Value) -> String {
        match self {
            JqTemplate::Mustache(mustache) => mustache.render(value),
            JqTemplate::JqTemplate(jq_transformer) => todo!(),
        }
    }

    pub fn render_graphql<'a, Ctx: ResolverContextLike>(&self, value: &EvalContext<'a, Ctx>) -> String {
        match self {
            JqTemplate::Mustache(mustache) => mustache.render_graphql(value),
            JqTemplate::JqTemplate(jq_transformer) => {
                let Some(value) = value.value() else {
                    return String::default()
                };
                jq_transformer.render_graphql(value)
            },
        }
    }
}

pub struct JqTransformer {
    filter: Filter<Native<Val>>,
    inputs: RcIter<Empty<Result<Val, String>>>,
    terms: Vec<Term<String>,
}

impl JqTransformer {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &str) -> Result<Self, JqTemplateError> {
        // the template is used to be parsed in to the IR AST
        let template = File { code: template, path: () };
        // defs is used to extend the syntax with custom definitions of functions, like 'toString'
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
        // the hardcoded inputs for the AST
        let inputs = RcIter::new(core::iter::empty());

        Ok(Self { filter, inputs })
    }

    /// Used to execute the transformation of the JQTemplate
    pub fn run<'a>(&'a self, data: Val) -> impl Iterator<Item = ValR<Val>> + 'a {
        let ctx = Ctx::new([], &self.inputs);
        self.filter.run((ctx, data))
    }

    pub fn render(&self, value: &serde_json::Value) -> String {
        self.render_helper(Val::from(value.clone()))
    }

    pub fn render_graphql(&self, value: &async_graphql_value::ConstValue) -> String {
        let Ok(value) = value.clone().into_json() else {
            return String::default()
        };

        self.render_helper(Val::from(value))
    }

    fn render_helper(&self, value: Val) -> String {
        let res = self.run(value);
        res.filter_map(|v| {
            if let Ok(v) = v {
                Some(v)
            } else {
                None
            }
        }).fold(String::new(), |acc, cur| {
            let cur_string = cur.to_string();
            acc + &cur_string
        })
    }

    /// Used to determine if the expression can be supported with current Mustache implementation
    pub fn is_select_operation(template: &str) -> bool {
        let lexer = jaq_core::load::Lexer::new(template);
        let lex = lexer.lex().unwrap_or_default();
        let mut parser = jaq_core::load::parse::Parser::new(&lex);
        let term = parser.term().unwrap_or_default();
        Self::recursive_is_select_operation(term)
    }

    /// Used as a helper function to determine if the term can be supported with Mustache implementation
    fn recursive_is_select_operation(term: jaq_core::load::parse::Term<&str>) -> bool {
        match term {
            jaq_core::load::parse::Term::Id => true,
            jaq_core::load::parse::Term::Recurse => false,
            jaq_core::load::parse::Term::Num(_) => false,
            jaq_core::load::parse::Term::Str(formater, _) => formater.is_none(),
            jaq_core::load::parse::Term::Arr(_) => false,
            jaq_core::load::parse::Term::Obj(_) => false,
            jaq_core::load::parse::Term::Neg(_) => false,
            jaq_core::load::parse::Term::Pipe(local_term_1, pattern, local_term_2) => {
                if pattern.is_some() {
                    false
                } else {
                    Self::recursive_is_select_operation(*local_term_1)
                        && Self::recursive_is_select_operation(*local_term_2)
                }
            }
            jaq_core::load::parse::Term::BinOp(_, _, _) => false,
            jaq_core::load::parse::Term::Label(_, _) => false,
            jaq_core::load::parse::Term::Break(_) => false,
            jaq_core::load::parse::Term::Fold(_, _, _, _) => false,
            jaq_core::load::parse::Term::TryCatch(_, _) => false,
            jaq_core::load::parse::Term::IfThenElse(_, _) => false,
            jaq_core::load::parse::Term::Def(_, _) => false,
            jaq_core::load::parse::Term::Call(_, _) => false,
            jaq_core::load::parse::Term::Var(_) => false,
            jaq_core::load::parse::Term::Path(local_term, path) => {
                Self::recursive_is_select_operation(*local_term)
                    && Self::is_path_select_operation(path)
            }
        }
    }

    fn is_path_select_operation(
        path: jaq_core::path::Path<jaq_core::load::parse::Term<&str>>,
    ) -> bool {
        path.0.into_iter().all(|part| match part {
            (jaq_core::path::Part::Index(idx), jaq_core::path::Opt::Optional) => Self::recursive_is_select_operation(idx),
            (jaq_core::path::Part::Index(idx), jaq_core::path::Opt::Essential) => Self::recursive_is_select_operation(idx),
            (jaq_core::path::Part::Range(_, _), jaq_core::path::Opt::Optional) => false,
            (jaq_core::path::Part::Range(_, _), jaq_core::path::Opt::Essential) => false,
        })
    }
}

impl Default for JqTransformer {
    fn default() -> Self {
        let inputs = RcIter::new(core::iter::empty());
        Self { filter: Default::default(), inputs }
    }
}

impl std::fmt::Debug for JqTransformer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTemplateData").finish()
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
        assert!(JqTransformer::is_select_operation(template), "Should return true for simple property access");
    }

    #[test]
    fn test_is_select_operation_nested_property() {
        let template = ".fruit.name";
        assert!(JqTransformer::is_select_operation(template), "Should return true for nested property access");
    }

    #[test]
    fn test_is_select_operation_array_index() {
        let template = ".fruits[1]";
        assert!(!JqTransformer::is_select_operation(template), "Should return false for array index access");
    }

    #[test]
    fn test_is_select_operation_pipe_operator() {
        let template = ".fruits[] | .name";
        assert!(!JqTransformer::is_select_operation(template), "Should return false for pipe operator usage");
    }

    #[test]
    fn test_is_select_operation_filter() {
        let template = ".fruits[] | select(.price > 1)";
        assert!(!JqTransformer::is_select_operation(template), "Should return false for select filter usage");
    }

    #[test]
    fn test_is_select_operation_function_call() {
        let template = "map(.price)";
        assert!(!JqTransformer::is_select_operation(template), "Should return false for function call");
    }
}
