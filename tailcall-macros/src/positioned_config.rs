extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Data;

#[derive(Debug)]
enum FieldType {
    OptionField,
    Field,
    Unknown,
}

fn get_field_type(attrs: &[syn::Attribute]) -> syn::Result<FieldType> {
    let mut field_type = FieldType::Unknown;
    for attr in attrs {
        if attr.path().is_ident("positioned_field") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("option_field") {
                    field_type = FieldType::OptionField;
                    Ok(())
                } else if meta.path.is_ident("field") {
                    field_type = FieldType::Field;
                    Ok(())
                } else {
                    Err(syn::Error::new(attr.span(), "Unknown helper attribute."))
                }
            })?;
        }
    }

    Ok(field_type)
}

pub fn expand_positoned_config(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_identifier = &input.ident;

    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let fields_matches: Vec<_> = fields
                .iter()
                .filter_map(|f| {
                    let field = &f.ident;
                    let field_name = field.as_ref().unwrap().to_string();
                    let attrs = &f.attrs;

                    match get_field_type(attrs) {
                        Ok(FieldType::OptionField) => Some(quote! {
                            #field_name => {
                                if let Some(ref mut positioned_field) = self.#field {
                                    positioned_field.set_position(position.0, position.1);
                                }
                            }
                        }),
                        Ok(FieldType::Field) => Some(quote! {
                            #field_name => {
                                self.#field.set_position(position.0, position.1);
                            }
                        }),
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
