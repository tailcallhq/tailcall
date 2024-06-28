extern crate proc_macro;

use proc_macro::TokenStream;

mod document_definition;
mod merge_right;
use crate::document_definition::expand_document_definition;
use crate::merge_right::expand_merge_right_derive;

#[proc_macro_derive(MergeRight, attributes(merge_right))]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    expand_merge_right_derive(input)
}

#[proc_macro_derive(DocumentDefinition, attributes(doc_type))]
pub fn document_definition_derive(input: TokenStream) -> TokenStream {
    expand_document_definition(input)
}
