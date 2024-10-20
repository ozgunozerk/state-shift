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

    // Parse the `state_slots` and `default_state` from the arguments
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
    let generics = &input_struct.generics; // I added this line

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

    // Construct the new generics by merging original generics with state slots set to the default state
    let original_generics = generics.params.iter();
    let combined_generics = if generics.params.is_empty() {
        quote! {
            #(#state_idents = #default_generics),*
        }
    } else {
        quote! {
            #(#original_generics),*, #(#state_idents = #default_generics),*
        }
    };

    let where_clauses = (0..state_slots).map(|i| {
        let state_num = Ident::new(&format!("State{}", i + 1), struct_name.span());
        quote!(#state_num: TypeStateProtector)
    });

    // TODO: also append original where clause by the `where_clauses` above

    // Construct the `_state` field with PhantomData
    // `_state: PhantomData<fn() -> T>`
    // the reason for using `fn() -> T` is to: https://github.com/ozgunozerk/state-shift/issues/1
    let phantom_fields = state_idents
        .iter()
        .map(|ident| quote!(::std::marker::PhantomData<fn() -> #ident>))
        .collect::<Vec<_>>();

    let output = quote! {
        #[allow(clippy::type_complexity)]
        struct #struct_name<#combined_generics>
        where
            #(#where_clauses),* // TODO: probably this line as well
        {
            #struct_fields
            _state: (#(#phantom_fields),*),
        }
    };

    output.into()
}
