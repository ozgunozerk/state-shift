use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, Ident, ItemFn, PathArguments,
    ReturnType, Token, Type,
};

pub fn switch_to_inner(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input arguments and function: (State1, State2, ...)
    let args_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let parsed_args = args_parser.parse(args).unwrap();
    let input_fn = parse_macro_input!(input as ItemFn);

    // Get the function name, inputs, and body
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;

    // Get the full list of arguments as a vec: (A, B, State1, ...)
    let generic_idents: Vec<proc_macro2::TokenStream> =
        parsed_args.iter().map(|i| quote!(#i)).collect();

    // Parse the original return type from the function signature
    let original_return_type = match &input_fn.sig.output {
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

    // Construct the new method with the modified return type
    let output = quote! {
        fn #fn_name(#fn_inputs) -> #modified_return_type {
            #fn_body
        }
    };

    output.into()
}
