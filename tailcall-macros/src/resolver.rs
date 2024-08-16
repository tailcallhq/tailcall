use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn expand_resolver_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = if let Data::Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let ty = match &variant.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
                    _ => panic!("Resolver variants must have exactly one unnamed field"),
                };

                (variant_name, ty)
            })
            .collect::<Vec<_>>()
    } else {
        panic!("Resolver can only be derived for enums");
    };

    let variant_parsers = variants.iter().map(|(variant_name, ty)| {
        quote! {
            let temp_result = <#ty>::from_directives(directives.iter());
            let result_option = temp_result.to_result();
            if let Ok(Some(resolver)) = result_option {
                let directive_name = <#ty>::trace_name();
                if !resolvable_directives.contains(&directive_name) {
                    resolvable_directives.push(directive_name);
                }
                if result.is_some() {
                    return Valid::fail(format!(
                        "Multiple resolvers detected [{}]",
                        resolvable_directives.join(", ")
                    ));
                }
                result = Some(Self::#variant_name(resolver));
            } else if let Err(e) = result_option {
                return Valid::from_validation_err(e);
            }
        }
    });

    let match_arms_to_directive = variants.iter().map(|(variant_name, _ty)| {
        quote! {
            Self::#variant_name(d) => d.to_directive(),
        }
    });

    let match_arms_directive_name = variants.iter().map(|(variant_name, ty)| {
        quote! {
            Self::#variant_name(_) => <#ty>::directive_name(),
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn from_directives(
                directives: &[Positioned<ConstDirective>],
            ) -> Valid<Option<Self>, String> {
                let mut result = None;
                let mut resolvable_directives = Vec::new();

                #(#variant_parsers)*

                match result {
                    Some(res) => Valid::succeed(Some(res)),
                    None => Valid::succeed(None),
                }
            }

            pub fn to_directive(&self) -> ConstDirective {
                match self {
                    #(#match_arms_to_directive)*
                }
            }

            pub fn directive_name(&self) -> String {
                match self {
                    #(#match_arms_directive_name)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
