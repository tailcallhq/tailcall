extern crate proc_macro;

use proc_macro::TokenStream;

mod document_definition;
mod merge_right;
use crate::document_definition::{expand_directive_definition, expand_input_definition};
use crate::merge_right::expand_merge_right_derive;
#[proc_macro_derive(MergeRight, attributes(merge_right))]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    expand_merge_right_derive(input)
}

#[proc_macro_derive(DirectiveDefinition, attributes(directive_definition))]
pub fn directive_definitions_derive(input: TokenStream) -> TokenStream {
    expand_directive_definition(input)
}

#[proc_macro_derive(InputDefinition)]
pub fn input_definition_derive(input: TokenStream) -> TokenStream {
    expand_input_definition(input)
}
