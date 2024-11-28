use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

use crate::extract_idents_from_group;

pub fn type_state_inner(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input struct
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let generics = &input_struct.generics;
    let visibility = &input_struct.vis;

    // Parse arguments (states and slots)
    /*
    Indices:
    ---
    0. `states`
    1. `=`
    2. `(State1, State2, State3)`
    3. `,`
    4. `slots`
    5. `=`
    6. `(State1, State1)`
     */
    let input_args: Vec<_> = args.into_iter().collect();
    let states: Vec<Ident> =
        extract_idents_from_group(&input_args[2], struct_name, "expected a list of states");

    let default_slots: Vec<Ident> = extract_idents_from_group(
        &input_args[6],
        struct_name,
        "expected a list of default slots",
    );

    // Generate the marker structs and sealing traits
    let sealer_trait_name = Ident::new(&format!("Sealer{}", struct_name), struct_name.span());
    let sealed_mod_name = Ident::new(
        &format!("sealed_{}", to_snake_case(&struct_name.to_string())),
        struct_name.span(),
    );

    let markers: Vec<_> = states
        .iter()
        .map(|state| {
            let marker_name = Ident::new(&format!("{}", state), state.span());
            quote! {
                pub struct #marker_name;
            }
        })
        .collect();

    let sealed_impls: Vec<_> = states
        .iter()
        .map(|state| {
            let marker_name = Ident::new(&format!("{}", state), state.span());
            quote! {
                impl #sealed_mod_name::Sealed for #marker_name {}
            }
        })
        .collect();

    let trait_impls: Vec<_> = states
        .iter()
        .map(|state| {
            let marker_name = Ident::new(&format!("{}", state), state.span());
            quote! {
                impl #sealer_trait_name for #marker_name {}
            }
        })
        .collect();

    // Extract fields from the struct
    // we cannot use `input_struct.fields` directly because
    // quote! treats the Fields reference as a block expression,
    // leading to the generated fields being wrapped inside
    // an extra set of braces ({ ... }).
    let struct_fields = match input_struct.fields {
        Fields::Named(ref fields) => &fields.named,
        Fields::Unnamed(_) => panic!("Expected named fields in struct."),
        Fields::Unit => panic!("Expected a struct with fields."),
    };

    // Generate state generics: `struct StructName<PlayerState1, PlayerState2, ...>`
    let state_idents: Vec<_> = (0..default_slots.len())
        .map(|i| {
            Ident::new(
                &format!("{}State{}", struct_name, i + 1),
                struct_name.span(),
            )
        })
        .collect();

    // Construct the new generics by merging original generics with default states
    let default_generics = default_slots.iter().collect::<Vec<_>>();
    let combined_generics = if generics.params.is_empty() {
        quote! { #(#state_idents = #default_generics),* }
    } else {
        let original_generics = generics.params.iter();
        quote! { #(#original_generics),*, #(#state_idents = #default_generics),* }
    };

    // create a new where clause for the new generics (states)
    let new_where_clause: Vec<_> = state_idents
        .iter()
        .map(|state| quote!(#state: #sealer_trait_name))
        .collect();

    // Merge the where clauses if there is an existing one
    let merged_where_clause = if let Some(existing_where) = &generics.where_clause {
        quote! { #existing_where #(#new_where_clause),* }
    } else if !new_where_clause.is_empty() {
        quote! { where #(#new_where_clause),* }
    } else {
        quote! {}
    };

    // Construct the `_state` field with PhantomData
    // `_state: PhantomData<fn() -> T>`
    // the reason for using `fn() -> T` is to: https://github.com/ozgunozerk/state-shift/issues/1
    let phantom_fields = state_idents
        .iter()
        .map(|ident| quote!(::core::marker::PhantomData<fn() -> #ident>))
        .collect::<Vec<_>>();

    // Get the struct's attributes (other macros) excluding the #[type_state] macro
    let attrs: Vec<_> = input_struct
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("type_state"))
        .collect();

    // Generate the final output
    let output = quote! {
        mod #sealed_mod_name {
            pub trait Sealed {}
        }

        pub trait #sealer_trait_name: #sealed_mod_name::Sealed {}

        #(#markers)*

        #(#sealed_impls)*

        #(#trait_impls)*

        #(#attrs)*
        #[allow(clippy::type_complexity)]
        #visibility struct #struct_name<#combined_generics>
        #merged_where_clause
        {
            #struct_fields
            _state: (#(#phantom_fields),*),
        }
    };

    output.into()
}
