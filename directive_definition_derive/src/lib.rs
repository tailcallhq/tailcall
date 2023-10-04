use proc_macro::TokenStream;

use quote::quote;
use syn::{parse, Data, DataStruct, Fields, Type};

#[proc_macro_derive(DirectiveDefinition)]
pub fn directive_definition_derive(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = parse(input).unwrap();
  impl_directive_definition(&ast)
}

fn get_first_seg_ident_string(path: &syn::TypePath) -> Option<String> {
    if let Some(seg) = path.path.segments.first() {
        Some(seg.ident.to_string())
    } else {
        None
    }
}

// fn convert_to_graphql_type(ident: String, arg: Option<String>) -> String {
//     println!("{:?}", arg);
//     let default_type = "String".to_string();
//     match ident.as_str() {
//         "Option" => format!("{}!!", convert_to_graphql_type(arg.unwrap_or(default_type), None)),
//         "BTreeMap" => "[KeyValue]".to_string(),
//         "Vec" => format!("[{}]", convert_to_graphql_type(arg.unwrap_or(default_type), None)),
//         "Method" => "Method".to_string(),
//         _ => format!("{}", ident)
//     }
// }

fn get_graphql_type(path: &syn::TypePath) -> String {
    let ident_string = get_first_seg_ident_string(path);
    let argument_types = match &path.path.segments.first().unwrap().arguments {
        syn::PathArguments::AngleBracketed(angle_bracketed_args) => {
            angle_bracketed_args.args.iter().filter_map(|arg|
                match arg {
                    syn::GenericArgument::Type(syn::Type::Path(arg_type_path)) => {
                        Some(get_graphql_type(&arg_type_path))
                    },
                    _ => None
                }
            ).collect::<Vec<String>>().join(", ")
        },
        _ => "".to_string()
    };
    let mut graphql_type_str = match ident_string.as_ref().unwrap().as_str() {
        "Option" => format!("{}!!", argument_types),
        "BTreeMap" => "[KeyValue]".to_string(),
        "Vec" => format!("[{}]", argument_types),
        "Method" => "Method".to_string(),
        _ => {
            format!("{}", ident_string.unwrap())
        }

    };
    // TODO if ends with !!, remove, else add !
    if graphql_type_str.ends_with("!!") {
        graphql_type_str = graphql_type_str.strip_suffix("!!").unwrap_or("").to_string();
    } else {
        graphql_type_str.push_str("!")
    }
    graphql_type_str

}

// fn get_graphql_type_for_arg() -> String {

// }

fn impl_directive_definition(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let directive_name = name.to_string().to_lowercase();

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let field_names = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            stringify!(#field_name),
        }
    }).collect::<Vec<_>>();

    let field_types = fields.iter().map(|field| {
        let field_type = &field.ty;
        let graphql_type = match field_type {
            Type::Path(path) => {
                get_graphql_type(path)
            },
            _ => "String".to_string()
        };
        // let graphql_type = match field_type {
        //     Type::Path(path) => {
        //         let ident_string = get_first_seg_ident_string(path);
        //         let mut first_arg: Option<String> = None;
        //         if let Some(seg) = path.path.segments.first() {                
        //             first_arg = match &seg.arguments {
        //                 syn::PathArguments::AngleBracketed(angle_bracketed_args) => {
        //                     let first = angle_bracketed_args.args.first();
        //                     if let Some(arg_path) = first {
        //                         match arg_path {
        //                             syn::GenericArgument::Type(t) => {
        //                                 match t {
        //                                     syn::Type::Path(type_path) => {
        //                                         get_first_seg_ident_string(type_path)
        //                                     }
        //                                     _ => None
        //                                 }
        //                             }
        //                             _ => None
        //                         }
        //                     } else {
        //                         None
        //                     }
        //                 },
        //                 _ => None
        //             };
        //         }
        //         convert_to_graphql_type(ident_string.unwrap_or("".to_string()), first_arg)
        //     },
        //     _ => "String".to_string()
        // };

        quote! {
            #graphql_type,
        }
    }).collect::<Vec<_>>();

    
    let gen = quote! {
        impl #name {
            fn directive_definition() -> async_graphql::parser::types::DirectiveDefinition {
                let arguments = vec![#(#field_names)*].iter().zip(vec![#(#field_types)*].iter()).map(|(field_name, field_type)| {
                    let default_value = match field_type.to_string().as_str() {
                        "Method" => Some(async_graphql::Positioned::new(
                            async_graphql_value::ConstValue::String("GET".to_string()), 
                            async_graphql::Pos::default()
                        )),
                        _ => None
                    };

                    async_graphql::Positioned::new(
                        async_graphql::parser::types::InputValueDefinition { 
                            description: None,
                            name: async_graphql::Positioned::new(
                                async_graphql_value::Name::new(field_name), 
                                async_graphql::Pos::default()
                            ),
                            ty: async_graphql::Positioned::new(
                                async_graphql::parser::types::Type::new(field_type).unwrap(),
                                async_graphql::Pos::default()
                            ),
                            default_value,
                            directives: Vec::new()
                        },
                        async_graphql::Pos::default()
                    )
                }).collect::<Vec<_>>();
                async_graphql::parser::types::DirectiveDefinition {
                    name: async_graphql::Positioned::new(
                        async_graphql_value::Name::new(#directive_name), 
                        async_graphql::Pos::default()
                    ),
                    description: None,
                    arguments,
                    is_repeatable: false,
                    locations: Vec::new(),
                }
            }
        }   
    };
    gen.into()
}

