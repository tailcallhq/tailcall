extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

const MERGE_RIGHT_FN: &str = "merge_right_fn";
const MERGE_RIGHT: &str = "merge_right";

#[derive(Default)]
struct Attrs {
    merge_right_fn: Option<syn::ExprPath>,
}

fn get_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attrs> {
    let mut attrs_ret = Attrs::default();
    for attr in attrs {
        if attr.path().is_ident(MERGE_RIGHT) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(MERGE_RIGHT_FN) {
                    let p: syn::Expr = meta.value()?.parse()?;
                    let lit =
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit), .. }) = p {
                            let suffix = lit.suffix();
                            if !suffix.is_empty() {
                                return Err(syn::Error::new(
                                    lit.span(),
                                    format!("unexpected suffix `{}` on string literal", suffix),
                                ));
                            }
                            lit
                        } else {
                            return Err(syn::Error::new(
                                p.span(),
                                format!(
                                    "expected merge_right {} attribute to be a string.",
                                    MERGE_RIGHT_FN
                                ),
                            ));
                        };
                    let expr_path: syn::ExprPath = lit.parse()?;
                    attrs_ret.merge_right_fn = Some(expr_path);
                    Ok(())
                } else {
                    Err(syn::Error::new(attr.span(), "Unknown helper attribute."))
                }
            })?;
        }
    }
    Ok(attrs_ret)
}

pub fn expand_merge_right_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();
    let generics = input.generics.clone();
    let gen = match input.data {
        // Implement for structs
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => &fields.named,
                Fields::Unnamed(fields) => &fields.unnamed,
                Fields::Unit => {
                    return quote! {
                        impl crate::core::merge_right::MergeRight for #name {
                            fn merge_right(self, other: Self) -> crate::core::valid::Valid<Self, String> {
                                crate::core::valid::Valid::succeed(other)
                            }
                        }
                    }
                    .into()
                }
            };

            let merge_logic = fields.iter().enumerate().map(|(i, f)| {
                let attrs = get_attrs(&f.attrs).unwrap();
                let name = &f.ident;

                match &data.fields {
                    Fields::Named(_) | Fields::Unnamed(_) => {
                        let merge = if let Some(merge_right_fn) = attrs.merge_right_fn {
                            quote! {
                                #merge_right_fn(self.#name, other.#name)
                            }
                        } else {
                            quote! {
                                self.#name.merge_right(other.#name)
                            }
                        };

                        if i == 0 {
                            merge
                        } else {
                            quote! {
                                .fuse(#merge)
                            }
                        }
                    }
                    Fields::Unit => unreachable!(),
                }
            });

            let fields = fields.iter().enumerate().map(|(i, f)| match &f.ident {
                Some(name) => name.clone().to_token_stream(),
                None => {
                    let name = format_ident!("x{i}");
                    name.to_token_stream()
                }
            });
            let fields_initializer = fields.clone();

            let generics_lt = generics.lt_token;
            let generics_gt = generics.gt_token;
            let generics_params = generics.params;

            let generics_del = quote! {
                #generics_lt #generics_params #generics_gt
            };

            let initializer = match data.fields {
                Fields::Named(_) => quote! {
                    Self {
                        #(#fields),*
                    }
                },
                Fields::Unnamed(_) => quote! {
                    Self(#(#fields),*)
                },
                Fields::Unit => unreachable!(),
            };

            quote! {
                impl #generics_del crate::core::merge_right::MergeRight for #name #generics_del {
                    fn merge_right(self, other: Self) -> crate::core::valid::Valid<Self, String> {
                        use crate::core::valid::Validator;

                        #(#merge_logic)*
                        .map(|(#(#fields_initializer),*)| {
                            #initializer
                        })
                    }
                }
            }
        }
        // Implement for enums
        Data::Enum(_) => quote! {
            impl crate::core::merge_right::MergeRight for #name {
                fn merge_right(self, other: Self) -> crate::core::valid::Valid<Self, String> {
                    crate::core::valid::Valid::succeed(other)
                }
            }
        },
        // Optionally handle or disallow unions
        Data::Union(_) => {
            return syn::Error::new_spanned(input, "Union types are not supported by MergeRight")
                .to_compile_error()
                .into()
        }
    };

    gen.into()
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Attribute};

    use super::*;

    #[test]
    fn test_get_attrs_invalid_type() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[merge_right(merge_right_fn = 123)])];
        let result = get_attrs(&attrs);
        assert!(
            result.is_err(),
            "Expected error with non-string type for `merge_right_fn`"
        );
    }

    #[test]
    fn test_get_attrs_unexpected_suffix() {
        let attrs: Vec<Attribute> =
            vec![parse_quote!(#[merge_right(merge_right_fn = "some_fn()")])];
        let result = get_attrs(&attrs);
        assert!(
            result.is_err(),
            "Expected error with unexpected suffix on string literal"
        );
    }
}
