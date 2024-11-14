use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use tailcall_valid::Valid;

use crate::core::document::print_directives;
use crate::core::ir::model::{IO, IR};
use crate::core::jit::{Field, OperationPlan};
use crate::core::json::JsonLikeOwned;
use crate::core::{Mustache, Transform};

#[derive(Default)]
pub struct GraphQL<A>(PhantomData<A>);

impl<A> GraphQL<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A: Display + Debug + JsonLikeOwned> Transform for GraphQL<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        for field in plan.selection.iter_mut() {
            if let Some(IR::IO(IO::GraphQL { req_template, .. })) = field.ir.as_mut() {
                if let Some(v) = format_selection_set(field.selection.iter()) {
                    req_template.selection = Some(Mustache::parse(&v).into());
                }
            }
        }

        Valid::succeed(plan)
    }
}

fn format_selection_set<'a, A: 'a + Display + JsonLikeOwned>(
    selection_set: impl Iterator<Item = &'a Field<A>>,
) -> Option<String> {
    let set = selection_set
        .filter(|field| !matches!(&field.ir, Some(IR::IO(_)) | Some(IR::Dynamic(_))))
        .map(|field| {
            // handle @modify directive scenario.
            let field_name = if let Some(IR::ContextPath(data)) = &field.ir {
                data.first().cloned().unwrap_or(field.name.to_string())
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

fn format_selection_field<A: Display + JsonLikeOwned>(field: &Field<A>, name: &str) -> String {
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

fn format_selection_field_arguments<A: Display>(field: &Field<A>) -> Cow<'static, str> {
    let arguments = field
        .args
        .iter()
        .filter(|a| a.value.is_some())
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(",");

    if arguments.is_empty() {
        Cow::Borrowed("")
    } else {
        Cow::Owned(format!("({})", arguments.escape_default()))
    }
}
