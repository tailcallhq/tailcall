use proc_macro::TokenStream;

use quote::quote;
use syn::{parse, Data};

#[proc_macro_derive(EnumDefinition)]
pub fn enum_definition_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = parse(input).unwrap();
    impl_enum_definition(&ast)
}

fn impl_enum_definition(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let enum_name = name.to_string();
    let variants = match &ast.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return quote! {
                compile_error!("enum definition derive macro can only be used with enums");
            }
            .into();
        }
    };
    let variant_names = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            stringify!(#variant_name),
        }
    }).collect::<Vec<_>>();

    let gen = quote! {
        impl #name {
            fn enum_definition() -> async_graphql::parser::types::TypeDefinition {
                let values = vec![#(#variant_names)*].iter().map(|variant_name| {
                    async_graphql::Positioned::new(
                        async_graphql::parser::types::EnumValueDefinition {
                            description: None,
                            value: async_graphql::Positioned::new(
                                    async_graphql_value::Name::new(variant_name), 
                                    async_graphql::Pos::default()
                                ),
                            directives: Vec::new()
                        },
                        async_graphql::Pos::default()
                    )
                }).collect::<Vec<_>>();

                let kind = async_graphql::parser::types::TypeKind::Enum(
                    async_graphql::parser::types::EnumType {
                        values
                    }
                );

                let name = async_graphql::Positioned::new(
                    async_graphql_value::Name::new(#enum_name),
                    async_graphql::Pos::default()
                );
                
                async_graphql::parser::types::TypeDefinition {
                    extend: false,
                    description: None,
                    name,
                    directives: Vec::new(),
                    kind,
                }

            }
        }
    };
    gen.into()
}