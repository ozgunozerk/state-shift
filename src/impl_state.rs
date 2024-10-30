use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl, Type};

use crate::{extract_macro_args, generate_impl_block_for_method_based_on_require_args};

pub fn impl_state_inner(item: TokenStream) -> TokenStream {
    // Parse the impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Extract the type name of the impl block (e.g., Player)
    let struct_name = match *input.self_ty {
        Type::Path(ref type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type for impl block"),
    };

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

    // Generate the expanded code with unique modules and traits
    let expanded = quote! {
        #(#methods)*
    };

    expanded.into()
}
