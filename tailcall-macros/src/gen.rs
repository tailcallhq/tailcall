use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Meta};

pub fn doc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let variants = if let Data::Enum(data_enum) = input.data {
        data_enum.variants
    } else {
        panic!("Doc can only be used on enums");
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let docs = variant
            .attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let Meta::NameValue(value) = &attr.meta {
                        if let Expr::Lit(lit) = &value.value {
                            if let syn::Lit::Str(lit_str) = &lit.lit {
                                return Some(lit_str.value().trim().to_string());
                            }
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>()
            .join("\n");

        quote! {
            #name::#variant_name => #docs.to_string(),
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn doc(&self) -> String {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
