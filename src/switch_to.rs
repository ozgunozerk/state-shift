use syn::{punctuated::Punctuated, Ident, PathArguments, ReturnType, Token, Type, TypePath};

pub fn switch_to_inner(
    fn_output: &ReturnType,
    parsed_args: &Punctuated<Ident, Token![,]>,
    struct_name: &Ident, // New parameter for the struct name (e.g., Player)
) -> ReturnType {
    let generic_idents: Vec<syn::GenericArgument> = parsed_args
        .iter()
        .map(|i| {
            syn::GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: i.clone().into(),
            }))
        })
        .collect();

    let original_return_type = match &fn_output {
        ReturnType::Type(_, ty) => &**ty,
        _ => panic!("Expected a return type."),
    };

    let mut modified_return_type = original_return_type.clone();

    // Recursively modify the return type, using the struct_name to match
    recursively_modify_return_type(&mut modified_return_type, generic_idents, struct_name);

    ReturnType::Type(Default::default(), Box::new(modified_return_type))
}

fn recursively_modify_return_type(
    ty: &mut Type,
    generic_idents: Vec<syn::GenericArgument>,
    struct_name: &Ident, // Target struct name (e.g., Player)
) {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last_mut().unwrap();

            if last_segment.ident == *struct_name {
                // Match the provided struct name (e.g., Player)
                modify_type_path(type_path, generic_idents);
            } else {
                // Handle cases where it's a wrapper type like Result<Player> or Option<Player>
                match &mut last_segment.arguments {
                    PathArguments::AngleBracketed(arguments) => {
                        for arg in &mut arguments.args {
                            if let syn::GenericArgument::Type(inner_type) = arg {
                                // Recurse into the inner type to check for Player (or any target struct)
                                recursively_modify_return_type(
                                    inner_type,
                                    generic_idents.clone(),
                                    struct_name,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => panic!("Expected a path type."),
    }
}

fn modify_type_path(type_path: &mut TypePath, generic_idents: Vec<syn::GenericArgument>) {
    let last_segment = type_path.path.segments.last_mut().unwrap();

    match &mut last_segment.arguments {
        PathArguments::AngleBracketed(arguments) => {
            arguments.args.extend(generic_idents);
        }
        PathArguments::None => {
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
}
