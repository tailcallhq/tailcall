use super::{Mustache, Segment};
use crate::core::path::{PathGraphql, PathString};

pub trait Eval<'a> {
    type In;
    type Out;

    fn eval(&'a self, mustache: &'a Mustache, in_value: &'a Self::In) -> Self::Out;
}

pub struct PathStringEval<A>(std::marker::PhantomData<A>);

impl<A> PathStringEval<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'a, A: PathString> Eval<'a> for PathStringEval<A> {
    type In = A;
    type Out = String;

    fn eval(&self, mustache: &Mustache, in_value: &Self::In) -> Self::Out {
        mustache
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.clone(),
                Segment::Expression(parts) => in_value
                    .path_string(parts)
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

pub trait Path {
    fn get_path<S: AsRef<str>>(&self, in_value: &[S]) -> Option<&Self>;
}

pub struct PathEval<A>(std::marker::PhantomData<A>);

impl<A> PathEval<A> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[allow(unused)]
pub enum Exit<'a, A> {
    Text(&'a str),
    Value(&'a A),
}

impl<'a, A: Path + 'a> Eval<'a> for PathEval<&'a A> {
    type In = &'a A;
    type Out = Vec<Exit<'a, A>>;

    fn eval(&'a self, mustache: &'a Mustache, in_value: &'a Self::In) -> Self::Out {
        mustache
            .segments()
            .iter()
            .filter_map(|segment| match segment {
                Segment::Literal(text) => Some(Exit::Text(text)),
                Segment::Expression(parts) => in_value.get_path(parts).map(Exit::Value),
            })
            .collect::<Vec<_>>()
    }
}

pub struct PathGraphqlEval<A>(std::marker::PhantomData<A>);

impl<A> PathGraphqlEval<A> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'a, A: PathGraphql> Eval<'a> for PathGraphqlEval<A> {
    type In = A;
    type Out = String;

    fn eval(&self, mustache: &Mustache, in_value: &Self::In) -> Self::Out {
        mustache
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.to_string(),
                Segment::Expression(parts) => in_value.path_graphql(parts).unwrap_or_default(),
            })
            .collect()
    }
}

impl Mustache {
    // TODO: drop these methods and directly use the eval implementations
    pub fn render(&self, value: &impl PathString) -> String {
        PathStringEval::new().eval(self, value)
    }

    pub fn render_graphql(&self, value: &impl PathGraphql) -> String {
        PathGraphqlEval::new().eval(self, value)
    }
}

#[cfg(test)]
mod tests {

    mod render {
        use std::borrow::Cow;

        use serde_json::json;

        use crate::core::mustache::{Mustache, Segment};
        use crate::core::path::PathString;

        #[test]
        fn test_query_params_template() {
            let s = r"/v1/templates?project-id={{value.projectId}}";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            let ctx = json!(json!({"value": {"projectId": "123"}}));
            let result = mustache.render(&ctx);
            assert_eq!(result, "/v1/templates?project-id=123");
        }

        #[test]
        fn test_render_mixed() {
            struct DummyPath;

            impl PathString for DummyPath {
                fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some(Cow::Borrowed("FOOBAR"))
                    } else if parts == ["baz", "qux"] {
                        Some(Cow::Borrowed("BAZQUX"))
                    } else {
                        None
                    }
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" middle ".to_string()),
                Segment::Expression(vec!["baz".to_string(), "qux".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(
                mustache.render(&DummyPath),
                "prefix FOOBAR middle BAZQUX suffix"
            );
        }

        #[test]
        fn test_render_with_missing_path() {
            struct DummyPath;

            impl PathString for DummyPath {
                fn path_string<T: AsRef<str>>(&self, _: &[T]) -> Option<Cow<'_, str>> {
                    None
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(mustache.render(&DummyPath), "prefix  suffix");
        }

        #[test]
        fn test_json_like() {
            let mustache =
                Mustache::parse(r#"{registered: "{{foo}}", display: "{{bar}}"}"#).unwrap();
            let ctx = json!({"foo": "baz", "bar": "qux"});
            let result = mustache.render(&ctx);
            assert_eq!(result, r#"{registered: "baz", display: "qux"}"#);
        }

        #[test]
        fn test_json_like_static() {
            let mustache = Mustache::parse(r#"{registered: "foo", display: "bar"}"#).unwrap();
            let ctx = json!({}); // Context is not used in this case
            let result = mustache.render(&ctx);
            assert_eq!(result, r#"{registered: "foo", display: "bar"}"#);
        }

        #[test]
        fn test_render_preserves_spaces() {
            struct DummyPath;

            impl PathString for DummyPath {
                fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo"] {
                        Some(Cow::Borrowed("bar"))
                    } else {
                        None
                    }
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("    ".to_string()),
                Segment::Expression(vec!["foo".to_string()]),
                Segment::Literal("    ".to_string()),
            ]);

            assert_eq!(mustache.render(&DummyPath).as_str(), "    bar    ");
        }
    }

    mod render_graphql {
        use crate::core::mustache::{Mustache, Segment};
        use crate::core::path::PathGraphql;

        #[test]
        fn test_render_mixed() {
            struct DummyPath;

            impl PathGraphql for DummyPath {
                fn path_graphql<T: AsRef<str>>(&self, parts: &[T]) -> Option<String> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some("FOOBAR".to_owned())
                    } else if parts == ["baz", "qux"] {
                        Some("BAZQUX".to_owned())
                    } else {
                        None
                    }
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" middle ".to_string()),
                Segment::Expression(vec!["baz".to_string(), "qux".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(
                mustache.render_graphql(&DummyPath),
                "prefix FOOBAR middle BAZQUX suffix"
            );
        }

        #[test]
        fn test_render_with_missing_path() {
            struct DummyPath;

            impl PathGraphql for DummyPath {
                fn path_graphql<T: AsRef<str>>(&self, _: &[T]) -> Option<String> {
                    None
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(mustache.render_graphql(&DummyPath), "prefix  suffix");
        }
    }
}
