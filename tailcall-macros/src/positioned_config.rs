extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Data;

#[derive(Debug)]
enum FieldType {
    Field,
    OptionField,
    ListField,
    Unknown,
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Default)]
struct PositionedAttr {
    renamed_field: Option<String>,
    field_type: FieldType,
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<PositionedAttr> {
    let mut parsed_attr: PositionedAttr = Default::default();
    for attr in attrs {
        if attr.path().is_ident("positioned_field") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("input_source_name") {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = expr {
                        parsed_attr.renamed_field = Some(lit_str.value());
                    } else {
                        return Err(syn::Error::new(
                            expr.span(),
                            "expected input_field_name to be a string.".to_string(),
                        ));
                    }
                } else if meta.path.is_ident("option_field") {
                    parsed_attr.field_type = FieldType::OptionField;
                } else if meta.path.is_ident("field") {
                    parsed_attr.field_type = FieldType::Field;
                } else if meta.path.is_ident("list_field") {
                    parsed_attr.field_type = FieldType::ListField;
                } else {
                    return Err(syn::Error::new(attr.span(), "Unknown helper attribute."));
                }

                Ok(())
            })?;
        }
    }

    Ok(parsed_attr)
}

pub fn expand_positoned_config(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_identifier = &input.ident;

    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let fields_matches: Vec<_> = fields
                .iter()
                .filter_map(|f| {
                    let field = f.ident.as_ref().unwrap();
                    let field_name = field.to_string();
                    let attrs = &f.attrs;
                    let field_name = field_name.to_case(Case::Camel);

                    match parse_attrs(attrs) {
                        Ok(PositionedAttr {
                            field_type: FieldType::OptionField,
                            renamed_field,
                        }) => {
                            let rename_or_field_name =
                                renamed_field.unwrap_or_else(|| field_name.clone());
                            Some(quote! {
                                #rename_or_field_name => {
                                    if let Some(ref mut positioned_field) = self.#field {
                                        positioned_field.set_position(position.0, position.1);
                                    }
                                }
                            })
                        }
                        Ok(PositionedAttr { field_type: FieldType::Field, renamed_field }) => {
                            let rename_or_field_name =
                                renamed_field.unwrap_or_else(|| field_name.clone());
                            Some(quote! {
                                #rename_or_field_name => {
                                    self.#field.set_position(position.0, position.1);
                                }
                            })
                        }
                        Ok(PositionedAttr { field_type: FieldType::ListField, renamed_field }) => {
                            let rename_or_field_name =
                                renamed_field.unwrap_or_else(|| field_name.clone());
                            Some(quote! {
                                #rename_or_field_name => {
                                    for field in self.#field.iter_mut() {
                                        field.set_position(position.0, position.1);
                                    }
                                }
                            })
                        }
                        _ => None,
                    }
                })
                .collect();

            let generated_code = quote! {
                impl PositionedConfig for #struct_identifier {
                    fn set_field_position(&mut self, field: &str, position: (usize, usize)) {
                        match field {
                            #(#fields_matches,)*
                            _ => {}
                        }
                    }
                }
            };

            TokenStream::from(generated_code)
        }

        _ => unimplemented!(),
    }
}
