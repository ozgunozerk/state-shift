use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, ItemStruct};

pub fn type_state_inner(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_args: Vec<_> = args.into_iter().collect();
    let state_slots: usize = input_args[2]
        .to_string()
        .parse()
        .expect("Expected a valid number for state_slots.");
    let default_state: Ident = match &input_args[6] {
        proc_macro::TokenTree::Ident(ident) => ident.clone(),
        _ => panic!("Expected an identifier for default_state."),
    };

    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let generics = &input_struct.generics;
    let visibility = &input_struct.vis;

    // Generate state identifiers and generics
    let state_idents: Vec<Ident> = (0..state_slots)
        .map(|i| Ident::new(&format!("{}_State{}", struct_name, i + 1), struct_name.span()))
        .collect();
    let default_generics = vec![quote!(#default_state); state_slots];

    let combined_generics = if generics.params.is_empty() {
        quote! { #(#state_idents = #default_generics),* }
    } else {
        let original_generics = generics.params.iter();
        quote! { #(#original_generics),*, #(#state_idents = #default_generics),* }
    };

    let where_clauses: Vec<_> = state_idents.iter()
        .map(|state| quote!(#state: TypeStateProtector))
        .collect();

    let merged_where_clause = if let Some(existing_where) = &generics.where_clause {
        quote! { #existing_where #(#where_clauses),* }
    } else if !where_clauses.is_empty() {
        quote! { where #(#where_clauses),* }
    } else {
        quote! {}
    };

    // Generate PhantomData fields for each state
    let phantom_fields = state_idents.iter()
        .map(|ident| quote!(::std::marker::PhantomData<fn() -> #ident>))
        .collect::<Vec<_>>();

    let output = quote! {
        #[allow(clippy::type_complexity)]
        #visibility struct #struct_name<#combined_generics>
        #merged_where_clause
        {
            #input_struct.fields
            _state: (#(#phantom_fields),*),
        }
    };

    output.into()
}

