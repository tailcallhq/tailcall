use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields};

const ATTR_NAMESPACE: &str = "resolver";
const ATTR_SKIP_DIRECTIVE: &str = "skip_directive";

#[derive(Default)]
struct Attrs {
    skip_directive: bool,
}

fn parse_attrs(attributes: &Vec<Attribute>) -> syn::Result<Attrs> {
    let mut result = Attrs::default();

    for attr in attributes {
        if attr.path().is_ident(ATTR_NAMESPACE) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(ATTR_SKIP_DIRECTIVE) {
                    result.skip_directive = true;

                    return Ok(());
                }

                Err(meta.error("unrecognized resolver attribute"))
            })?;
        }
    }

    Ok(result)
}

pub fn expand_resolver_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let variants = if let Data::Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let attrs = &variant.attrs;
                let ty = match &variant.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
                    _ => panic!("Resolver variants must have exactly one unnamed field"),
                };

                let attrs = parse_attrs(attrs)?;

                Ok((variant_name, ty, attrs))
            })
            .collect::<syn::Result<Vec<_>>>()?
    } else {
        panic!("Resolver can only be derived for enums");
    };

    let variant_parsers = variants.iter().filter_map(|(variant_name, ty, attrs)| {
        if attrs.skip_directive {
            return None;
        }

        Some(quote! {
            if <#ty>::directive_name() == directive.node.name.node {
                return <#ty>::from_directive(&directive.node).map(|x| Some(Self::#variant_name(x)))
            }
        })
    });

    let match_arms_to_directive = variants.iter().map(|(variant_name, _ty, attrs)| {
        if attrs.skip_directive {
            quote! {
                Self::#variant_name(d) => None,
            }
        } else {
            quote! {
                Self::#variant_name(d) => Some(d.to_directive()),
            }
        }
    });

    let match_arms_directive_name = variants.iter().map(|(variant_name, ty, attrs)| {
        if attrs.skip_directive {
            quote! {
                Self::#variant_name(_) => String::new(),
            }
        } else {
            quote! {
                Self::#variant_name(_) => <#ty>::directive_name(),
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn from_directives(
                directives: &[Positioned<ConstDirective>],
            ) -> Valid<crate::core::config::ResolverSet, String> {
                Valid::from_iter(directives.iter(), |directive| {
                    #(#variant_parsers)*

                    Valid::succeed(None)
                })
                    .map(|resolvers| {
                        crate::core::config::ResolverSet(resolvers.into_iter().flatten().collect())
                    })
            }

            pub fn to_directive(&self) -> Option<ConstDirective> {
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

    Ok(expanded)
}
