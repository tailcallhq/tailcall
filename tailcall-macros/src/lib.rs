extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(MergeRight)]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    let gen = match input.data {
        // Implement for structs
        Data::Struct(data) => {
            let fields = if let Fields::Named(fields) = data.fields {
                fields.named
            } else {
                // Adjust this match arm to handle other kinds of struct fields (unnamed/tuple
                // structs, unit structs)
                unimplemented!()
            };

            let merge_logic = fields.iter().map(|f| {
                let name = &f.ident;
                quote! {
                    #name: self.#name.merge_right(other.#name),
                }
            });
            
            let generics_lt = generics.lt_token;
            let generics_gt = generics.gt_token;
            let generics_params = generics.params;

            let generics_del = quote! {
                #generics_lt #generics_params #generics_gt
            };

            quote! {
                impl #generics_del MergeRight for #name #generics_del {
                    fn merge_right(self, other: Self) -> Self {
                        Self {
                            #(#merge_logic)*
                        }
                    }
                }
            }
        }
        // Implement for enums
        Data::Enum(_) => quote! {
            impl MergeRight for #name {
                fn merge_right(self, other: Self) -> Self {
                    other
                }
            }
        },
        // Optionally handle or disallow unions
        Data::Union(_) => unimplemented!(),
    };

    gen.into()
}
