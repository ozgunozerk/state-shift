use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ImplItem, ItemImpl, Token, Type,
};

use crate::{extract_require_args, generate_impl_block_for_method_based_on_require_args};

struct StatesInput {
    states: Punctuated<Ident, Token![,]>,
}

impl Parse for StatesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let states = Punctuated::parse_terminated(input)?;
        Ok(StatesInput { states })
    }
}

pub fn states_inner(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of states from the attribute: (State1, State2, ...)
    let args = parse_macro_input!(attr as StatesInput);

    // Parse the impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Extract the type name of the impl block (e.g., Player)
    let struct_name = match *input.self_ty {
        Type::Path(ref type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type for impl block"),
    };

    // Parse the generics and lifetimes associated with the impl block
    let generics = input.generics.clone();

    // Extract the methods from the impl block
    let mut methods = Vec::new();

    for item in input.items.iter_mut() {
        if let ImplItem::Fn(ref mut method) = item {
            // Extract `#[require]` arguments if they exist
            let require_args = extract_require_args(&mut method.attrs);

            // Generate the impl block for the method based on the extracted #[require] arguments
            let modified_method = if let Some(require_args) = require_args {
                generate_impl_block_for_method_based_on_require_args(
                    method,
                    &struct_name,
                    &require_args,
                    &generics,
                )
            } else {
                quote! { #method }
            };

            // Step 3: Push the modified method to the list of methods
            methods.push(modified_method);
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
