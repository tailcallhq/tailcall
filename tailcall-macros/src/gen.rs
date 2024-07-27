use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Meta};

fn extract_gen_doc_ty(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("gen_doc") {
                let meta_list = attr.meta.require_list().ok()?;
                let expr = meta_list.parse_args::<Expr>().ok()?;
                if let Expr::Assign(assign) = expr {
                    if let Expr::Path(expr_path) = assign.left.as_ref() {
                        let segment = expr_path.path.segments.first()?;
                        if segment.ident == "ty" {
                            if let Expr::Lit(expr_lit) = *assign.right {
                                if let syn::Lit::Str(lit_str) = expr_lit.lit {
                                    return Some(lit_str.value().trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("")
        .to_string()
}

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

    let match_arms_ty = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let ty = extract_gen_doc_ty(&variant.attrs).to_lowercase();

        let instance_type = match ty.as_str() {
            "integer" => quote! { InstanceType::Integer },
            "string" => quote! { InstanceType::String },
            "object" => quote! { InstanceType::Integer },
            _ => quote! { InstanceType::Null },
        };

        quote! {
            #name::#variant_name => #instance_type,
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn doc(&self) -> String {
                match self {
                    #(#match_arms)*
                }
            }
            pub fn ty(&self) -> InstanceType {
                match self {
                    #(#match_arms_ty)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
