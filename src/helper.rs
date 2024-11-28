use proc_macro::TokenTree;
use syn::{punctuated::Punctuated, Attribute, Ident, Token};

/// Helper function to find and remove an attribute by name
fn find_and_remove_attr(attrs: &mut Vec<Attribute>, attr_name: &str) -> Option<Attribute> {
    let pos = attrs
        .iter()
        .position(|attr| attr.path().is_ident(attr_name))?;
    Some(attrs.remove(pos))
}

/// Extracts the arguments from a macro call
pub fn extract_macro_args(
    attrs: &mut Vec<Attribute>,
    macro_name: &str,
) -> Option<Punctuated<Ident, Token![,]>> {
    let attr = find_and_remove_attr(attrs, macro_name)?;
    let args: Punctuated<Ident, Token![,]> =
        attr.parse_args_with(Punctuated::parse_terminated).ok()?;
    Some(args)
}

pub fn is_single_letter(ident: &Ident) -> bool {
    ident.to_string().len() == 1
}

pub fn extract_idents_from_group(token: &TokenTree, error_msg: &str) -> Vec<Ident> {
    match token {
        proc_macro::TokenTree::Group(group) => group
            .stream()
            .into_iter()
            .filter_map(|tt| {
                if let proc_macro::TokenTree::Ident(ident) = tt {
                    Some(Ident::new(&format!("{}", ident), ident.span().into()))
                } else {
                    None
                }
            })
            .collect(),
        _ => panic!("{}", error_msg),
    }
}
