use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

pub fn type_state_inner(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the `state_slots` and `default_state` from the arguments
    /*
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

    // Parse the input struct
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let generics = &input_struct.generics;
    let visibility = &input_struct.vis;

    // Parse the `state_slots` and `default_state` from the arguments
    let input_args: Vec<_> = args.into_iter().collect();
    let state_slots: usize = input_args[2]
        .to_string()
        .parse()
        .expect("Expected a valid number for state_slots.");
    let default_state: Ident = match &input_args[6] {
        proc_macro::TokenTree::Ident(ident) => {
            Ident::new(&format!("{}{}", struct_name, ident), ident.span().into())
        }
        _ => panic!("Expected an identifier for default_state."),
    };

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
    let state_idents: Vec<Ident> = (0..state_slots)
        .map(|i| {
            Ident::new(
                &format!("{}State{}", struct_name, i + 1),
                struct_name.span(),
            )
        })
        .collect();
    let default_generics = vec![quote!(#default_state); state_slots];

    // Construct the new generics by merging original generics with state slots set to the default state
    let combined_generics = if generics.params.is_empty() {
        quote! { #(#state_idents = #default_generics),* }
    } else {
        let original_generics = generics.params.iter();
        quote! { #(#original_generics),*, #(#state_idents = #default_generics),* }
    };

    let sealer_trait_name = Ident::new(&format!("Sealer{}", struct_name), struct_name.span());
    let where_clauses: Vec<_> = state_idents
        .iter()
        .map(|state| quote!(#state: #sealer_trait_name))
        .collect();

    let merged_where_clause = if let Some(existing_where) = &generics.where_clause {
        quote! { #existing_where #(#where_clauses),* }
    } else if !where_clauses.is_empty() {
        quote! { where #(#where_clauses),* }
    } else {
        quote! {}
    };

    // Construct the `_state` field with PhantomData
    // `_state: PhantomData<fn() -> T>`
    // the reason for using `fn() -> T` is to: https://github.com/ozgunozerk/state-shift/issues/1
    let phantom_fields = state_idents
        .iter()
        .map(|ident| quote!(::std::marker::PhantomData<fn() -> #ident>))
        .collect::<Vec<_>>();

    let output = quote! {
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
