use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Meta, Attribute};

/*fn extract_doc_type(attrs: &[Attribute]) -> String {
    attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("gen_doc") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.clone();
                println!("{:?}", tokens);
                let parsed: Expr = syn::parse2(tokens).unwrap();
                println!("{:?}", parsed);
                if let Expr::Assign(assign) = parsed {
                    if let Expr::Lit(lit) = assign.left.as_ref() {
                        if let syn::Lit::Str(lit_str) = &lit.lit {
                            if lit_str.value().eq("ty") {
                                if let Expr::Lit(lit) = assign.right.as_ref() {
                                    if let syn::Lit::Str(lit_str) = &lit.lit {
                                        return Some(lit_str.value().trim().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                /*parsed.iter().find_map(|expr| {
                    if let Expr::Lit(lit) = expr {
                        if let syn::Lit::Str(lit_str) = &lit.lit {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                    None::<String>
                });*/
            }
        }
        None::<String>
    }).collect::<Vec<_>>().join(" ")
}
*/
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
                if attr.path().is_ident("gen_doc") {
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

/*    let match_arms_ty = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let ty = extract_doc_type(&variant.attrs).to_lowercase();

        let instance_type = match ty.as_str() {
            "integer" => quote! { InstanceType::Integer },
            "string" => quote! { InstanceType::String },
            _ => quote! { InstanceType::Null },
        };

        quote! {
            #name::#variant_name => #instance_type,
        }
    });*/

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
