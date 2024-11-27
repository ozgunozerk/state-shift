/// this file contains the logic that modifies the methods that are annotated with `#[require]` macro,
/// however, all the functions inside this file will be used by `#[impl_state]` macro due to delegation needs
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, Expr, ExprStruct, GenericParam, Ident, ImplItemFn, Member, Stmt, Token,
    TypeParam,
};

use crate::{extract_macro_args, is_single_letter, switch_to_inner};

pub fn generate_impl_block_for_method_based_on_require_args(
    input_fn: &mut ImplItemFn,
    struct_name: &Ident,
    parsed_args: &Punctuated<Ident, Token![,]>,
    impl_generics: &syn::Generics,
    struct_generics: &syn::PathArguments,
) -> proc_macro2::TokenStream {
    // Convert the struct's generics into a Punctuated collection
    let mut combined_generics = match struct_generics {
        syn::PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed.args.clone(),
        syn::PathArguments::None => Punctuated::new(),
        _ => panic!("Unsupported generics format for struct"),
    };

    // Append the full list of arguments from `#[require]` macro: (A, B, State1, ...)
    combined_generics.extend(parsed_args.iter().map(|ident| {
        // Convert each parsed argument into a GenericArgument (which is a TypeParam)
        syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()), // Use the ident for the type path
        }))
    }));

    // put the sealed trait boundary for the generics:
    /*
    ``` where
    A: Sealer,
    B: Sealer,
     */
    let sealer_trait_name = Ident::new(&format!("Sealer{}", struct_name), struct_name.span());
    let new_where_clauses: Vec<proc_macro2::TokenStream> = parsed_args
        .iter()
        .filter(|ident| is_single_letter(ident))
        .map(|ident| quote!(#ident: #sealer_trait_name))
        .collect();

    // Merge with the existing where clause, if any.
    let merged_where_clause = if let Some(existing_where) = &impl_generics.where_clause {
        quote! {
            #existing_where #(#new_where_clauses),*
        }
    } else if !new_where_clauses.is_empty() {
        quote! {
            where #(#new_where_clauses),*
        }
    } else {
        quote! {}
    };

    // Merge the original generics with the new single-letter generics.
    let mut all_generics = impl_generics.params.clone();
    for ident in parsed_args.iter().filter(|i| is_single_letter(i)) {
        all_generics.push(GenericParam::Type(TypeParam::from(ident.clone())));
    }

    // Generate PhantomData for the required number of states
    let phantom_data: Vec<_> = (0..parsed_args.len())
        .map(|_| quote!(::core::marker::PhantomData))
        .collect();

    let phantom_expr = if phantom_data.len() == 1 {
        quote! { ::core::marker::PhantomData }
    } else {
        quote! { ( #(#phantom_data),* ) }
    };

    // Modify the function body to append `_state: (PhantomData, ...)` to struct fields.
    let new_fn_body: Vec<_> = input_fn
        .block
        .stmts
        .iter()
        .map(|stmt| {
            if let Stmt::Expr(expr, maybe_semi) = stmt {
                if let Some(modified_expr) =
                    modify_struct_in_expr(expr, struct_name, phantom_expr.clone())
                {
                    // Return the modified expression as a statement
                    return Stmt::Expr(modified_expr, *maybe_semi);
                }
            }
            stmt.clone()
        })
        .collect();

    // Collect other function attributes (excluding `#[require]`).
    let mut other_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("require"))
        .cloned()
        .collect();

    let fn_output = &input_fn.sig.output;
    let switch_to_args = extract_macro_args(&mut other_attrs, "switch_to", struct_name);

    // Generate the impl block for the method based on the extracted #[switch_to] arguments
    let new_output = if let Some(switch_to_args) = switch_to_args {
        switch_to_inner(fn_output, &switch_to_args, struct_name, &input_fn.sig.ident)
    } else {
        // there is no `#[switch_to]` macro, so we use the `#[require]` macro's arguments instead
        // to keep the type same for the input and the output
        switch_to_inner(fn_output, &parsed_args, struct_name, &input_fn.sig.ident)
    };

    // construct the signature again
    let fn_sig = &mut input_fn.sig;
    fn_sig.output = new_output;

    // extract visibility
    let fn_vis = &input_fn.vis;

    // Generate the final output `impl` block.
    let output = quote! {
        impl<#all_generics> #struct_name<#combined_generics>
        #merged_where_clause
        {
            #(#other_attrs)*
            #fn_vis #fn_sig {
                #(#new_fn_body)*
            }
        }
    };

    output
}

fn modify_struct_in_expr(
    expr: &Expr,
    struct_name: &syn::Ident,
    phantom_expr: TokenStream,
) -> Option<Expr> {
    match expr {
        Expr::Struct(expr_struct) if expr_struct.path.is_ident(struct_name) => {
            // Clone the struct fields and add the `_state` field
            let mut new_fields = expr_struct.fields.clone();
            new_fields.push(syn::FieldValue {
                attrs: Vec::new(),
                member: Member::Named(syn::Ident::new("_state", struct_name.span())),
                colon_token: Some(<Token![:]>::default()),
                expr: Expr::Verbatim(phantom_expr.clone()),
            });

            // Return a modified struct expression with the new fields
            Some(Expr::Struct(ExprStruct {
                fields: new_fields,
                ..expr_struct.clone()
            }))
        }
        // If it's an expression like `Some(Player { ... })` or `Ok(Player { ... })`
        Expr::Call(call_expr) => {
            let mut new_args = vec![];
            let mut modified = false;

            for arg in &call_expr.args {
                let phantom = phantom_expr.clone();
                if let Some(modified_arg) = modify_struct_in_expr(arg, struct_name, phantom) {
                    new_args.push(modified_arg);
                    modified = true;
                } else {
                    new_args.push(arg.clone());
                }
            }

            if modified {
                Some(Expr::Call(syn::ExprCall {
                    args: new_args.into_iter().collect(),
                    ..call_expr.clone()
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}
