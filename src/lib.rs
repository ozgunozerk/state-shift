extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Fields, Ident, ImplItem, ItemFn, ItemImpl, ItemStruct, Meta, ReturnType, Token, Type,
};

#[proc_macro_attribute]
pub fn require(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input arguments and function: (ImplType, State1, State2, ...)
    let args_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let parsed_args = args_parser.parse(args).unwrap();

    // Extract the first argument (the name of the impl block)

    let struct_name = &parsed_args[0];

    // Extract the remaining arguments (states and generics)

    let remaining_args: Vec<Ident> = parsed_args.iter().skip(1).cloned().collect();

    let input_fn = parse_macro_input!(input as ItemFn);

    // Only the single letter arguments will be used as generic constraints: (A, B, ...)
    let generic_idents: Vec<proc_macro2::TokenStream> = remaining_args
        .iter()
        .filter(|ident| is_single_letter(ident))
        .map(|ident| quote!(#ident))
        .collect();

    // Get the full list of arguments as a vec: (A, B, State1, ...)
    let concrete_type: Vec<proc_macro2::TokenStream> =
        remaining_args.iter().map(|ident| quote!(#ident)).collect();

    // put the sealed trait boundary for the generics:
    /*
    ``` where
    A: TypeStateProtector,
    B: TypeStateProtector,
     */
    let where_clauses: Vec<proc_macro2::TokenStream> = remaining_args
        .iter()
        .filter(|ident| is_single_letter(ident))
        .map(|ident| quote!(#ident: TypeStateProtector))
        .collect(); // Collect into a Vec to make `is_empty()` available

    // Generate the `where` clause only if there are any constraints
    let where_clause = if !where_clauses.is_empty() {
        quote! { where #(#where_clauses),* }
    } else {
        quote! {}
    };

    // Get the function name and its generics
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;

    // Collect all other macros except the `#[require]` attribute itself
    let other_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("require"))
        .collect();

    // Construct the `impl` block
    let output = quote! {
        impl<#(#generic_idents),*> #struct_name<#(#concrete_type),*>
        #where_clause
        {
            #(#other_attrs)*
            fn #fn_name(#fn_inputs) #fn_output {
                #fn_body
            }
        }
    };

    output.into()
}

// Helper function to determine if a path is a single-letter identifier
fn is_single_letter(ident: &Ident) -> bool {
    let ident_str = ident.to_string();
    ident_str.len() == 1
}

#[proc_macro_attribute]
pub fn switch_to(args: TokenStream, input: TokenStream) -> TokenStream {
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

struct StatesInput {
    states: Punctuated<Ident, Token![,]>,
}

impl Parse for StatesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let states = Punctuated::parse_terminated(input)?;
        Ok(StatesInput { states })
    }
}

#[proc_macro_attribute]
pub fn states(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of states from the attribute: (State1, State2, ...)
    let args = parse_macro_input!(attr as StatesInput);

    // Parse the impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Extract the type name of the impl block (e.g., Player)
    let impl_type = match *input.self_ty {
        Type::Path(ref type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type for impl block"),
    };

    // Extract the methods from the impl block
    let mut methods = Vec::new();

    for item in input.items.iter_mut() {
        if let ImplItem::Fn(ref mut method) = item {
            // Check if the method has a `#[require]` attribute
            for attr in method.attrs.iter_mut() {
                if attr.path().is_ident("require") {
                    // Parse the tokens of the `#[require]` macro
                    let mut args: Punctuated<Ident, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated).unwrap();

                    // Append the impl block type (e.g., Player) as the first argument
                    args.insert(0, impl_type.clone());

                    // Update the attribute tokens with the new arguments
                    let a = match attr.meta {
                        Meta::List(ref mut list) => list,
                        _ => panic!("Expected a list of arguments"),
                    };

                    a.tokens = quote! { #args };
                }
            }

            methods.push(quote! { #method });
        }
    }

    // Generate the marker structs, and their implementations
    let mut markers = Vec::new();
    let mut sealed_impls = Vec::new();
    let mut trait_impls = Vec::new();

    for state in args.states {
        let marker_name = Ident::new(&format!("{}", state), state.span());

        markers.push(quote! {
            struct #marker_name;
        });

        sealed_impls.push(quote! {
            impl sealed::Sealed for #marker_name {}
        });

        trait_impls.push(quote! {
            impl TypeStateProtector for #marker_name {}
        });
    }

    // Generate the full expanded code
    let expanded = quote! {
        // Private module to seal traits
        mod sealed {
            pub trait Sealed {}
        }

        pub trait TypeStateProtector: sealed::Sealed {}

        #(#markers)*

        #(#sealed_impls)*

        #(#trait_impls)*

        #(#methods)*
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn type_state(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the `state_slots` and `default_state` from the arguments
    /*
    Usage: `#[type_state(state_slots = 3, default_state = Initial)]`

    Indices:
    ---
    0. `state_slots`
    1. `=`
    2. `3` (this is the value you're interested in for state_slots)
    4. `,`
    5. `default_state`
    6. `=`
    7. `Initial` (this is the value you're interested in for default_state)
     */
    let input_args: Vec<_> = args.into_iter().collect();
    let state_slots: usize = if let Some(proc_macro::TokenTree::Literal(lit)) = input_args.get(2) {
        lit.to_string().parse().unwrap()
    } else {
        panic!("Expected a valid number for state_slots.");
    };

    let default_state: Ident = if let Some(proc_macro::TokenTree::Ident(ident)) = input_args.get(6)
    {
        Ident::new(&format!("{}", ident), ident.span().into())
    } else {
        panic!("Expected an identifier for default_state.");
    };

    // Parse the input struct
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    // Extract fields from the struct
    let struct_fields = match input_struct.fields {
        Fields::Named(ref fields) => &fields.named,
        Fields::Unnamed(_) => panic!("Expected named fields in struct."),
        Fields::Unit => panic!("Expected a struct with fields."),
    };

    // Generate state generics: `struct StructName<State1, State2, ...>`
    let state_idents: Vec<Ident> = (0..state_slots)
        .map(|i| Ident::new(&format!("State{}", i + 1), struct_name.span()))
        .collect();

    let default_generics = vec![quote!(#default_state); state_slots];

    let where_clauses = (0..state_slots).map(|i| {
        let state_num = Ident::new(&format!("State{}", i + 1), struct_name.span());
        quote!(#state_num: TypeStateProtector)
    });

    // Construct the `_state` field with PhantomData
    let phantom_fields = state_idents
        .iter()
        .map(|ident| quote!(PhantomData<#ident>))
        .collect::<Vec<_>>();

    let output = quote! {
        struct #struct_name<#(#state_idents = #default_generics),*>
        where
            #(#where_clauses),*
        {
            #struct_fields
            _state: (#(#phantom_fields),*),
        }
    };

    output.into()
}
