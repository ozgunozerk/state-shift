/// this file contains the logic that modifies the methods that are annotated with `#[require]` macro,
/// however, all the functions inside this file will be used by `#[states]` macro due to delegation needs
use quote::quote;
use syn::{
    punctuated::Punctuated, Expr, GenericParam, Ident, ImplItemFn, Member, Stmt, Token, TypeParam,
};

use crate::{extract_macro_args, is_single_letter, switch_to_inner};

pub fn generate_impl_block_for_method_based_on_require_args(
    input_fn: &mut ImplItemFn,
    struct_name: &Ident,
    parsed_args: &Punctuated<Ident, Token![,]>,
    impl_generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    // existing generics and lifetimes
    let mut combined_generics = impl_generics.params.clone();

    // Append the full list of arguments from `#[require]` macro: (A, B, State1, ...)
    combined_generics.extend(parsed_args.iter().map(|ident| {
        GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: ident.clone(),
            colon_token: None,
            bounds: Punctuated::new(),
            eq_token: None,
            default: None,
        })
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
        .map(|_| quote!(::std::marker::PhantomData))
        .collect();

    let phantom_expr = if phantom_data.len() == 1 {
        quote! { ::std::marker::PhantomData }
    } else {
        quote! { ( #(#phantom_data),* ) }
    };

    // Modify the function body to append `_state: (PhantomData, ...)` to struct fields.
    let new_fn_body: Vec<_> = input_fn
        .block
        .stmts
        .iter()
        .map(|stmt| {
            if let Stmt::Expr(Expr::Struct(expr_struct), maybe_semi) = stmt {
                if expr_struct.path.is_ident(struct_name) {
                    let mut new_fields = expr_struct.fields.clone();
                    new_fields.push(syn::FieldValue {
                        attrs: Vec::new(),
                        member: Member::Named(syn::Ident::new("_state", struct_name.span())),
                        colon_token: Some(<Token![:]>::default()),
                        expr: Expr::Verbatim(phantom_expr.clone()),
                    });
                    return Stmt::Expr(
                        syn::Expr::Struct(syn::ExprStruct {
                            fields: new_fields,
                            ..expr_struct.clone()
                        }),
                        *maybe_semi,
                    );
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
    let switch_to_args = extract_macro_args(&mut other_attrs, "switch_to", &struct_name);

    // Generate the impl block for the method based on the extracted #[require] arguments
    let new_output = if let Some(switch_to_args) = switch_to_args {
        switch_to_inner(fn_output, &switch_to_args)
    } else {
        fn_output.clone()
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
