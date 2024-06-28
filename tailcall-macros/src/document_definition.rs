extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn get_attr(input: &DeriveInput, attr_name: &str) -> String {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(attr_name))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().ok())
        .expect("Expected a doc_type attribute")
        .value()
}

pub fn expand_document_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_identifier = &input.ident;
    let doc_type = get_attr(&input, "doc_type");
    let expanded = match doc_type.as_str() {
        "Directive" => quote! {
            doc.definitions.push(Self::to_doc_directive(
                root_schema.clone(),
                stringify!(#struct_identifier),
            ));
        },
        "Scalar" => quote! {
            doc.definitions
                .push(Self::to_doc_scalar(root_schema, stringify!(#struct_identifier)));
        },
        "Input" => quote! {
                doc.definitions
                    .push(Self::to_doc_input(root_schema.clone(), stringify!(#struct_identifier)));
        },
        "DirectiveWithInput" => quote! {
            doc.definitions.push(Self::to_doc_directive(
                root_schema.clone(),
                stringify!(#struct_identifier),
            ));

            doc.definitions
                .push(Self::to_doc_input(root_schema.clone(), stringify!(#struct_identifier)));
        },
        _ => panic!(),
    };

    let expanded = quote! {
        impl tailcall_typedefs_common::DocumentDefinition for #struct_identifier {
            fn definition(doc: async_graphql::parser::types::ServiceDocument) -> async_graphql::parser::types::ServiceDocument {
                let mut doc = doc;
                let root_schema = schemars::schema_for!(Self);
                #expanded
                doc
            }
        }
    };

    TokenStream::from(expanded)
}
