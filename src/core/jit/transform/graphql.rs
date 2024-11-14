use std::{borrow::Cow, convert::Infallible, fmt::Debug, marker::PhantomData};

use async_graphql::{
    parser::types::{DirectiveDefinition, InputValueDefinition, Type},
    Name, Pos, Positioned,
};
use tailcall_valid::Valid;

use crate::core::{
    ir::model::{IO, IR},
    jit::{Directive, Field, OperationPlan},
    json::{JsonLike, JsonLikeOwned},
    Transform,
};

#[derive(Default)]
pub struct GraphQL<A>(PhantomData<A>);

impl<A> GraphQL<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A: ToString + Debug + JsonLikeOwned> Transform for GraphQL<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        for field in plan.selection.iter_mut() {
            // TODO: fix this.
            let rendered = field.render_graphql();
            if let Some(IR::IO(IO::GraphQL { req_template, .. })) = field.ir.as_mut() {
                req_template.selection = rendered;
            }
        }

        Valid::succeed(plan)
    }
}

impl<A: ToString + JsonLikeOwned> Field<A> {
    pub fn render_graphql(&self) -> Option<String> {
        format_selection_set(self.selection.iter())
    }
}

fn format_selection_set<'a, A: 'a + ToString + JsonLikeOwned>(
    selection_set: impl Iterator<Item = &'a Field<A>>,
) -> Option<String> {
    let set = selection_set
        .filter(|field| match &field.ir {
            Some(IR::IO(_)) | Some(IR::Dynamic(_)) => false,
            _ => true,
        })
        .map(|field| {
            // handle @modify directive scenario.
            let field_name = if let Some(IR::ContextPath(data)) = &field.ir {
                data.get(0).cloned().unwrap_or(field.name.to_string())
            } else {
                field.name.to_string()
            };
            format_selection_field(field, &field_name)
        })
        .collect::<Vec<_>>();

    if set.is_empty() {
        return None;
    }

    Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field<A: ToString + JsonLikeOwned>(field: &Field<A>, name: &str) -> String {
    let arguments = format_selection_field_arguments(field);
    let selection_set = format_selection_set(field.selection.iter());

    let mut output = format!("{}{}", name, arguments);

    if !field.directives.is_empty() {
        let directives = print_directives(field.directives.iter());

        if !directives.is_empty() {
            output.push(' ');
            output.push_str(&directives.escape_default().to_string());
        }
    }

    if let Some(selection_set) = selection_set {
        output.push(' ');
        output.push_str(&selection_set);
    }

    output
}

fn format_selection_field_arguments<A: ToString>(field: &Field<A>) -> Cow<'static, str> {
    let arguments = field
        .args
        .iter()
        .filter(|a| a.value.is_some())
        .collect::<Vec<_>>();

    if arguments.is_empty() {
        return Cow::Borrowed("");
    }

    let args = arguments
        .iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(",");

    Cow::Owned(format!("({})", args.escape_default()))
}

// TODO: refactor this.
pub fn print_directives<'a, A: 'a + JsonLikeOwned>(
    directives: impl Iterator<Item = &'a Directive<A>>,
) -> String {
    directives
        .map(|d| print_directive(&const_directive_to_sdl(d)))
        .collect::<Vec<String>>()
        .join(" ")
}

fn print_directive(directive: &DirectiveDefinition) -> String {
    let args = directive
        .arguments
        .iter()
        .map(|arg| format!("{}: {}", arg.node.name.node, arg.node.ty.node))
        .collect::<Vec<String>>()
        .join(", ");

    if args.is_empty() {
        format!("@{}", directive.name.node)
    } else {
        format!("@{}({})", directive.name.node, args)
    }
}

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn const_directive_to_sdl<Input: JsonLikeOwned>(
    directive: &Directive<Input>,
) -> DirectiveDefinition {
    let to_mustache = |s: &str| -> String {
        s.strip_prefix('$')
            .map(|v| format!("{{{{{}}}}}", v))
            .unwrap_or_else(|| s.to_string())
    };

    DirectiveDefinition {
        description: None,
        name: pos(Name::new(directive.name.as_str())),
        arguments: directive
            .arguments
            .iter()
            .filter_map(|(k, v)| {
                if !v.is_null() {
                    let v_str = to_mustache(&v.to_string_value());
                    Some(pos(InputValueDefinition {
                        description: None,
                        name: pos(Name::new(k)),
                        default_value: Some(pos(JsonLike::string(Cow::Borrowed(&v_str)))),
                        ty: pos(Type {
                            nullable: true,
                            base: async_graphql::parser::types::BaseType::Named(Name::new(v_str)),
                        }),
                        directives: Vec::new(),
                    }))
                } else {
                    None
                }
            })
            .collect(),
        is_repeatable: true,
        locations: vec![],
    }
}
