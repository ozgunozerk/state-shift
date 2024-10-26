use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ImplItem, ItemImpl, Token, Type,
};

use crate::{extract_macro_args, generate_impl_block_for_method_based_on_require_args};

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

    // Generate unique module and trait names by appending the struct name
    // Convert the struct name to snake case
    let sealed_mod_name = Ident::new(
        &format!("sealed_{}", to_snake_case(&struct_name.to_string())),
        struct_name.span(),
    );
    let sealer_trait_name = Ident::new(&format!("Sealer{}", struct_name), struct_name.span());

    // Extract the methods from the impl block
    let mut methods = Vec::new();

    for item in input.items.iter_mut() {
        if let ImplItem::Fn(ref mut method) = item {
            // Extract `#[require]` arguments if they exist
            let require_args = extract_macro_args(&mut method.attrs, "require", &struct_name);

            // Generate the impl block for the method based on the extracted #[require] arguments
            let modified_method = if let Some(require_args) = require_args {
                generate_impl_block_for_method_based_on_require_args(
                    method,
                    &struct_name,
                    &require_args,
                    &input.generics,
                )
            } else {
                quote! { #method }
            };

            // Step 3: Push the modified method to the list of methods
            methods.push(modified_method);
        }
    }

    // Generate marker structs, sealed implementations, and trait implementations
    let mut markers = Vec::new();
    let mut sealed_impls = Vec::new();
    let mut trait_impls = Vec::new();

    for state in args.states {
        let marker_name = Ident::new(&format!("{}{}", struct_name, state), state.span());

        markers.push(quote! {
            pub struct #marker_name;
        });

        sealed_impls.push(quote! {
            impl #sealed_mod_name::Sealed for #marker_name {}
        });

        trait_impls.push(quote! {
            impl #sealer_trait_name for #marker_name {}
        });
    }

    // Generate the expanded code with unique modules and traits
    let expanded = quote! {
        // Private module to seal traits (unique to this struct)
        mod #sealed_mod_name {
            pub trait Sealed {}
        }

        // Trait unique to this struct to ensure type state protection
        pub trait #sealer_trait_name: #sealed_mod_name::Sealed {}

        #(#markers)*

        #(#sealed_impls)*

        #(#trait_impls)*

        #(#methods)*
    };

    TokenStream::from(expanded)
}
