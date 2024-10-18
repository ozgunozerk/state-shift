use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, Ident, ItemFn, ReturnType, Token,
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
        ReturnType::Type(_, ty) => quote! { #ty },
        _ => panic!("Expected a return type."),
    };

    // Construct the new return type using the original name and the new generics
    let return_type = quote! {
        #original_return_type<#(#generic_idents),*>
    };

    // Construct the new method with the modified return type
    let output = quote! {
        fn #fn_name(#fn_inputs) -> #return_type {
            #fn_body
        }
    };

    output.into()
}
