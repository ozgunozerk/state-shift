use quote::quote;
use syn::{punctuated::Punctuated, Ident, PathArguments, ReturnType, Token, Type};

pub fn switch_to_inner(
    fn_output: &ReturnType,
    parsed_args: &Punctuated<Ident, Token![,]>,
) -> ReturnType {
    // Get the full list of arguments as a vec: (A, B, State1, ...)
    let generic_idents: Vec<proc_macro2::TokenStream> =
        parsed_args.iter().map(|i| quote!(#i)).collect();
    // Parse the original return type from the function signature
    let original_return_type = match &fn_output {
        ReturnType::Type(_, ty) => &**ty,
        _ => panic!("Expected a return type."),
    };

    // Check if the original return type has angle brackets for generics
    let modified_return_type = match original_return_type {
        Type::Path(type_path) => {
            // Extract the type path without generics (e.g., PlayerBuilder).
            let type_name = &type_path.path.segments.last().unwrap().ident;

            match &type_path.path.segments.last().unwrap().arguments {
                PathArguments::AngleBracketed(arguments) => {
                    // Extract existing generics.
                    let existing_generics = &arguments.args;
                    quote! {
                        #type_name<#existing_generics, #(#generic_idents),*>
                    }
                }
                PathArguments::None => {
                    // No existing generics, so we add ours as a new set.
                    quote! {
                        #type_name<#(#generic_idents),*>
                    }
                }
                _ => panic!("Unsupported path arguments in return type."),
            }
        }
        _ => panic!("Expected a return type that is a path."),
    };

    ReturnType::Type(Default::default(), Box::new(modified_return_type))
}
