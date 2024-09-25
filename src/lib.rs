extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Fields, Ident, ItemFn, ItemImpl, ItemStruct, Meta, Path, ReturnType, Token,
};

#[proc_macro_attribute]
pub fn require(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input arguments and function
    let args_parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed_args = args_parser.parse(args).unwrap();

    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract the states from the attribute arguments, converting single letters to `None`
    let state_constraints: Vec<Option<Path>> = parsed_args
        .iter()
        .map(|arg| {
            match arg {
                Meta::Path(path) => {
                    if is_single_letter(path) {
                        None // Single-letter argument, map to `None`
                    } else {
                        Some(path.clone()) // Not a single-letter, map to `Some`
                    }
                }
                _ => panic!("Invalid state argument format. Expected paths."),
            }
        })
        .collect();

    // Dynamically generate generic names (A, B, C, ...)
    let generic_idents: Vec<proc_macro2::TokenStream> = (0..state_constraints.len())
        .map(|i| {
            let generic_char = ('A' as u8 + i as u8) as char;
            let generic_ident =
                syn::Ident::new(&generic_char.to_string(), proc_macro2::Span::call_site());
            quote!(#generic_ident)
        })
        .collect();

    // Generate the `where` clause only for `Some` values in state_constraints
    let where_clauses = state_constraints
        .iter()
        .enumerate()
        .filter_map(|(i, opt_constraint)| {
            if let Some(constraint) = opt_constraint {
                let generic_ident = &generic_idents[i];
                Some(quote!(#generic_ident: #constraint))
            } else {
                None
            }
        });

    // Get the function name and its generics
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;

    // Extract the `switch_to` attributes to include them in the output
    let switch_to_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("switch_to"))
        .collect();

    // Construct the `impl` block
    let output = quote! {
        impl<#(#generic_idents),*> PlayerBuilder<#(#generic_idents),*>
        where
            #(#where_clauses),*
        {
            #(#switch_to_attrs)*
            fn #fn_name(#fn_inputs) #fn_output {
                #fn_body
            }
        }
    };

    output.into()
}

// Helper function to determine if a path is a single-letter identifier
fn is_single_letter(path: &Path) -> bool {
    if let Some(segment) = path.segments.first() {
        let ident_str = segment.ident.to_string();
        ident_str.len() == 1
    } else {
        false
    }
}

#[proc_macro_attribute]
pub fn switch_to(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input arguments and function
    let args_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let parsed_args = args_parser.parse(args).unwrap();
    let input_fn = parse_macro_input!(input as ItemFn);

    // Get the function name, inputs, and body
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;

    let generic_idents: Vec<proc_macro2::TokenStream> = parsed_args
        .iter()
        .map(|i| {
            if i.to_string().len() > 1 {
                let marker = Ident::new(&format!("{}Marker", i), i.span());
                quote!(#marker)
            } else {
                quote!(#i)
            }
        })
        .collect();

    // Parse the return type
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
    // Parse the list of states from the attribute
    let args = parse_macro_input!(attr as StatesInput);

    // Parse the impl block
    let input = parse_macro_input!(item as ItemImpl);

    // Extract the methods from the impl block
    let methods = input.items;

    // Generate the traits, markers, and their implementations
    let mut traits = Vec::new();
    let mut markers = Vec::new();
    let mut sealed_impls = Vec::new();
    let mut trait_impls = Vec::new();

    for state in args.states {
        let trait_name = Ident::new(&format!("{}", state), state.span());
        let marker_name = Ident::new(&format!("{}Marker", state), state.span());

        traits.push(quote! {
            pub trait #trait_name: sealed::Sealed {}
        });

        markers.push(quote! {
            struct #marker_name;
        });

        sealed_impls.push(quote! {
            impl sealed::Sealed for #marker_name {}
        });

        trait_impls.push(quote! {
            impl #trait_name for #marker_name {}
        });
    }

    // Generate the full expanded code
    let expanded = quote! {
        // Private module to seal traits
        mod sealed {
            pub trait Sealed {}
        }

        #(#traits)*

        #(#markers)*

        #(#sealed_impls)*

        #(#trait_impls)*

        #(#methods)*
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn type_state(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the state_slots and default_state from the arguments
    let input_args: Vec<_> = args.into_iter().collect();
    let state_slots: usize = if let Some(proc_macro::TokenTree::Literal(lit)) = input_args.get(2) {
        lit.to_string().parse().unwrap()
    } else {
        panic!("Expected a valid number for state_slots.");
    };

    let default_state: Ident = if let Some(proc_macro::TokenTree::Ident(ident)) = input_args.get(6)
    {
        Ident::new(&format!("{}Marker", ident), ident.span().into())
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

    // Generate state generics
    let state_idents: Vec<Ident> = (0..state_slots)
        .map(|i| Ident::new(&format!("State{}", i + 1), struct_name.span()))
        .collect();

    let default_generics = vec![quote!(#default_state); state_slots];

    // Construct the _state field with PhantomData
    let phantom_fields = state_idents
        .iter()
        .map(|ident| quote!(PhantomData<#ident>))
        .collect::<Vec<_>>();

    let output = quote! {
        struct #struct_name<#(#state_idents = #default_generics),*> {
            #struct_fields
            _state: (#(#phantom_fields),*),
        }
    };

    output.into()
}
