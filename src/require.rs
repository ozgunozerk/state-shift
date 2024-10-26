/// this file contains the logic that modifies the methods that are annotated with `#[require]` macro,
/// however, all the functions inside this file will be used by `#[states]` macro due to delegation needs
use quote::quote;
use syn::{
    punctuated::Punctuated, Attribute, Expr, GenericParam, Ident, ImplItemFn, Member, Stmt, Token,
    TypeParam,
};

pub fn extract_require_args(
    attrs: &mut Vec<Attribute>,
    struct_name: &Ident,
) -> Option<Punctuated<Ident, Token![,]>> {
    let pos = attrs
        .iter()
        .position(|attr| attr.path().is_ident("require"))?;
    let attr = attrs.remove(pos);

    // Parse the arguments from the `#[require]` macro
    let args: Punctuated<Ident, Token![,]> =
        attr.parse_args_with(Punctuated::parse_terminated).ok()?;

    // Modify the arguments: prefix struct name for non-single-letter idents
    let modified_args = args
        .into_iter()
        .map(|ident| {
            if is_single_letter(&ident) {
                ident
            } else {
                Ident::new(&format!("{}{}", struct_name, ident), ident.span())
            }
        })
        .collect::<Punctuated<Ident, Token![,]>>();

    Some(modified_args)
}

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
    let other_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("require"))
        .collect();

    // Get the function name and its generics
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_vis = &input_fn.vis;

    // Generate the final output `impl` block.
    let output = quote! {
        impl<#all_generics> #struct_name<#combined_generics>
        #merged_where_clause
        {
            #(#other_attrs)*
            #fn_vis fn #fn_name(#fn_inputs) #fn_output {
                #(#new_fn_body)*
            }
        }
    };

    output
}

fn is_single_letter(ident: &Ident) -> bool {
    ident.to_string().len() == 1
}
