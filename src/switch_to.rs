use syn::{punctuated::Punctuated, Ident, PathArguments, ReturnType, Token, Type, TypePath};

pub fn switch_to_inner(
    fn_output: &ReturnType,
    parsed_args: &Punctuated<Ident, Token![,]>,
) -> ReturnType {
    // Get the full list of arguments as syn::GenericArgument (A, B, State1, ...)
    let generic_idents: Vec<syn::GenericArgument> = parsed_args
        .iter()
        .map(|i| {
            syn::GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: i.clone().into(),
            }))
        })
        .collect();

    // Parse the original return type from the function signature
    let original_return_type = match &fn_output {
        ReturnType::Type(_, ty) => &**ty,
        _ => panic!("Expected a return type."),
    };

    // Check if the original return type has angle brackets for generics
    let modified_return_type = match original_return_type {
        Type::Path(type_path) => {
            // Clone the type_path to modify its segments
            let mut modified_type_path = type_path.clone();
            let last_segment = modified_type_path.path.segments.last_mut().unwrap(); // Mutable reference to the last segment

            match &mut last_segment.arguments {
                PathArguments::AngleBracketed(arguments) => {
                    // Add the new generics to existing generics
                    arguments.args.extend(generic_idents);
                }
                PathArguments::None => {
                    // No existing generics, so we add ours as a new set.
                    last_segment.arguments =
                        PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            args: generic_idents.into_iter().collect(),
                            colon2_token: None,
                            lt_token: Default::default(),
                            gt_token: Default::default(),
                        });
                }
                _ => panic!("Unsupported path arguments in return type."),
            }

            // Return the modified type
            Type::Path(modified_type_path)
        }
        _ => panic!("Expected a return type that is a path."),
    };

    // Return the modified ReturnType
    ReturnType::Type(Default::default(), Box::new(modified_return_type))
}
