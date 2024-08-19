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
            valid = valid.and(<#ty>::from_directives(directives.iter()).map(|resolver| {
                if let Some(resolver) = resolver {
                    let directive_name = <#ty>::trace_name();
                    if !resolvable_directives.contains(&directive_name) {
                        resolvable_directives.push(directive_name);
                    }
                    result = Some(Self::#variant_name(resolver));
                }
            }));
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
                let mut valid = Valid::succeed(());

                #(#variant_parsers)*

                valid.and_then(|_| {
                    if resolvable_directives.len() > 1 {
                        Valid::fail(format!(
                            "Multiple resolvers detected [{}]",
                            resolvable_directives.join(", ")
                        ))
                    } else {
                        Valid::succeed(result)
                    }
                })
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
