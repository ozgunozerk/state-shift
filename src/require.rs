/// this file contains the logic that modifies the methods that are annotated with `#[require]` macro,
/// however, all the functions inside this file will be used by `#[states]` macro due to delegation needs
use quote::quote;
use syn::{punctuated::Punctuated, Attribute, Expr, Ident, ImplItemFn, Member, Stmt, Token};

pub fn extract_require_args(attrs: &mut Vec<Attribute>) -> Option<Punctuated<Ident, Token![,]>> {
    let pos = attrs
        .iter()
        .position(|attr| attr.path().is_ident("require"))?;
    let attr = attrs.remove(pos);

    // Parse the arguments from the `#[require]` macro
    attr.parse_args_with(Punctuated::parse_terminated).ok()
}

pub fn generate_impl_block_for_method_based_on_require_args(
    input_fn: &mut ImplItemFn,
    struct_name: &Ident,
    parsed_args: &Punctuated<Ident, Token![,]>,
) -> proc_macro2::TokenStream {
    // Only the single letter arguments will be used as generic constraints: (A, B, ...)
    let generic_idents: Vec<proc_macro2::TokenStream> = parsed_args
        .iter()
        .filter(|ident| is_single_letter(ident))
        .map(|ident| quote!(#ident))
        .collect();

    // Get the full list of arguments as a vec: (A, B, State1, ...)
    let concrete_type: Vec<proc_macro2::TokenStream> =
        parsed_args.iter().map(|ident| quote!(#ident)).collect();

    // put the sealed trait boundary for the generics:
    /*
    ``` where
    A: TypeStateProtector,
    B: TypeStateProtector,
     */
    let where_clauses: Vec<proc_macro2::TokenStream> = parsed_args
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

    // Generate PhantomData for the required number of states
    let phantom_data_count = parsed_args.len();
    let phantom_data: Vec<proc_macro2::TokenStream> = (0..phantom_data_count)
        .map(|_| quote!(::std::marker::PhantomData))
        .collect();

    let phantom_expr = if phantom_data.len() == 1 {
        quote! { ::std::marker::PhantomData }
    } else {
        quote! { ( #(#phantom_data),* ) }
    };

    // Convert the method body to modify struct construction
    let new_fn_body = fn_body
        .stmts
        .iter()
        .map(|stmt| {
            // Check if the statement contains the struct initialization (e.g., `PlayerBuilder {`)
            if let Stmt::Expr(Expr::Struct(expr_struct), maybe_semi) = stmt {
                let struct_path = &expr_struct.path;
                if struct_path.is_ident(struct_name) {
                    // Append `_state: (PhantomData, PhantomData, ...)` to the struct fields
                    let mut new_fields = expr_struct.fields.clone();
                    new_fields.push(syn::FieldValue {
                        attrs: Vec::new(),
                        member: Member::Named(syn::Ident::new("_state", struct_name.span())),
                        colon_token: Some(<Token![:]>::default()),
                        expr: Expr::Verbatim(phantom_expr.clone()),
                    });

                    // Return modified struct construction
                    return Stmt::Expr(
                        syn::Expr::Struct(syn::ExprStruct {
                            fields: new_fields,
                            ..expr_struct.clone()
                        }),
                        *maybe_semi,
                    );
                }
            }
            // Return the statement unchanged if it's not a struct construction
            stmt.clone()
        })
        .collect::<Vec<_>>();

    // Construct the `impl` block
    let output = quote! {
        impl<#(#generic_idents),*> #struct_name<#(#concrete_type),*>
        #where_clause
        {
            #(#other_attrs)*
            fn #fn_name(#fn_inputs) #fn_output {
                #(#new_fn_body)*
            }
        }
    };

    output
}

fn is_single_letter(ident: &Ident) -> bool {
    let ident_str = ident.to_string();
    ident_str.len() == 1
}
