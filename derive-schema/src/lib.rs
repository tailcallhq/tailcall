use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;

  let expanded = match ast.data {
    syn::Data::Struct(ref s) => {
      let fields = match s.fields {
        syn::Fields::Named(ref fields) => fields
          .named
          .iter()
          .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            quote! {
               fields.insert(stringify!(#name).to_owned(), <#ty as Schema>::schema());
            }
          })
          .collect::<Vec<_>>(),
        _ => panic!("Only named fields are supported"),
      };

      quote! {
         impl Schema for #name {
            fn schema() -> DynamicSchema {
               DynamicSchema::Record {
                  name: stringify!(#name).to_owned(),
                  fields: {
                    let mut fields = BTreeMap::new();
                    #(#fields)*
                     fields
                }
              }
            }
         }
      }
    }
    _ => panic!("Only structs and enums are supported"),
  };

  TokenStream::from(expanded)
}
