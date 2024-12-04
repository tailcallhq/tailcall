use std::sync::Arc;

use jaq_core::{
    load::{parse::Term, Arena, File, Loader},
    Compiler, Ctx, Filter, Native, RcIter, ValR,
};
use jaq_json::Val;

use crate::core::ir::{EvalContext, ResolverContextLike};

use super::{Mustache, Segment};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum JqTemplate {
    Mustache(Mustache),
    JqTemplate(Arc<JqTransformer>)
}

impl JqTemplate {
    pub fn render(&self, value: &serde_json::Value) -> String {
        match self {
            JqTemplate::Mustache(mustache) => mustache.render(value),
            JqTemplate::JqTemplate(jq_transformer) => jq_transformer.render(value),
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

    pub fn is_const(&self) -> bool {
        match self {
            JqTemplate::Mustache(mustache) => mustache.is_const(),
            JqTemplate::JqTemplate(jq_transformer) => jq_transformer.is_const(),
        }
    }

    pub fn segments(&self) -> &Vec<Segment> {
        if let JqTemplate::Mustache(mustache) = self {
            mustache.segments()
        } else {
            unimplemented!()
        }
    }

    pub fn segments_mut(&mut self) -> &mut Vec<Segment> {
        if let JqTemplate::Mustache(mustache) = self {
            mustache.segments_mut()
        } else {
            unimplemented!()
        }
    }

    pub fn expression_segments(&self) -> Vec<&Vec<String>> {
        if let JqTemplate::Mustache(mustache) = self {
            mustache.expression_segments()
        } else {
            unimplemented!()
        }
    }

    pub fn expression_contains(&self, expression: &str) -> bool {
        if let JqTemplate::Mustache(mustache) = self {
            mustache.expression_contains(expression)
        } else {
            unimplemented!()
        }
    }

    pub fn parse(template: &str) -> Self {
        Self::Mustache(Mustache::parse(template))
    }
}

impl Default for JqTemplate {
    fn default() -> Self {
        Self::Mustache(Mustache::default())
    }
}

pub struct JqTransformer {
    filter: Filter<Native<Val>>,
    term: Term<&'static str>,
}

impl JqTransformer {
    /// Used to parse a `template` and try to convert it into a JqTemplate
    pub fn try_new(template: &'static str) -> Result<Self, JqTemplateError> {
        // the term is used because it can be easily serialized, deserialized and hashed
        let term = Self::parse_template(template);

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

        Ok(Self { filter, term })
    }

    /// Used to execute the transformation of the JQTemplate
    pub fn run<'a, T: std::iter::Iterator<Item = std::result::Result<Val, std::string::String>>>(&'a self, inputs: &'a RcIter<T>,data: Val) -> impl Iterator<Item = ValR<Val>> + 'a {
        let ctx = Ctx::new([], inputs);
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
        // the hardcoded inputs for the AST
        let inputs = RcIter::new(core::iter::empty());
        let res = self.run(&inputs, value);
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
        let term = Self::parse_template(template);
        Self::recursive_is_select_operation(term)
    }

    fn parse_template(template: &str) -> Term<&str> {
        let lexer = jaq_core::load::Lexer::new(template);
        let lex = lexer.lex().unwrap_or_default();
        let mut parser = jaq_core::load::parse::Parser::new(&lex);
        parser.term().unwrap_or_default()
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

    /// Used to determine if the transformer is a static value
    pub fn is_const(&self) -> bool {
        // TODO: parse terms to determine the value
        false
    }
}

impl Default for JqTransformer {
    fn default() -> Self {
        Self { filter: Default::default(), term: Term::default() }
    }
}

impl std::fmt::Debug for JqTransformer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JqTransformer").field("term", &self.term).finish()
    }
}

impl ToString for JqTransformer {
    fn to_string(&self) -> String {
        format!("[JqTransformer]({:?})", self.term)
    }
}

impl std::cmp::PartialEq for JqTransformer {
    fn eq(&self, other: &Self) -> bool {
        // TODO: sorry for the quick hack
        format!("{:?}", self).eq(&format!("{:?}", other))
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
