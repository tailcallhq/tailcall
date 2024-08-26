extern crate proc_macro;

use proc_macro::TokenStream;

mod document_definition;
mod gen;
mod merge_right;
mod resolver;

use crate::document_definition::{expand_directive_definition, expand_input_definition};
use crate::merge_right::expand_merge_right_derive;
use crate::resolver::expand_resolver_derive;

#[proc_macro_derive(MergeRight, attributes(merge_right))]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    expand_merge_right_derive(input)
}

#[proc_macro_derive(DirectiveDefinition, attributes(directive_definition))]
pub fn directive_definitions_derive(input: TokenStream) -> TokenStream {
    expand_directive_definition(input)
}

#[proc_macro_derive(Doc, attributes(gen_doc))]
pub fn scalar_definition_derive(input: TokenStream) -> TokenStream {
    gen::doc(input)
}

#[proc_macro]
pub fn gen_doc(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let name = &input.ident;
    let gen = quote::quote! {
        impl #name {
            pub fn doc() -> &'static str {
                stringify!(#name)
            }
        }
    };
    TokenStream::from(quote::quote! {
        #input
        #gen
    })
}

#[proc_macro_derive(InputDefinition)]
pub fn input_definition_derive(input: TokenStream) -> TokenStream {
    expand_input_definition(input)
}

#[proc_macro_derive(CustomResolver)]
pub fn resolver_derive(input: TokenStream) -> TokenStream {
    expand_resolver_derive(input)
}
