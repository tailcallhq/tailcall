extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::Data;

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

                    let should_position = attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("is_positioned_option"));

                    if should_position {
                        Some(quote! {
                            #field_name => {
                                if let Some(ref mut positioned_field) = self.#field {
                                    positioned_field.set_position(position.0, position.1);
                                }
                            }
                        })
                    } else {
                        None
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
