use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ImplItem, ItemImpl, Meta, Token, Type,
};

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
