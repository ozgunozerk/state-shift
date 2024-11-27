use syn::{
    punctuated::Punctuated, visit_mut::VisitMut, Ident, PathArguments, ReturnType, Token, Type,
    TypePath,
};

pub fn switch_to_inner(
    fn_output: &ReturnType,
    parsed_args: &Punctuated<Ident, Token![,]>,
    struct_name: &Ident,
    fn_name: &Ident,
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
        _ => panic!(
            "Function `{}`: Expected a return type with explicit type annotation (e.g., '-> Type'), but found none.",
            fn_name
        ),
    };

    let mut modified_return_type = original_return_type.clone();

    // Recursively modify the return type, using the struct_name to match
    recursively_modify_return_type(
        &mut modified_return_type,
        generic_idents,
        struct_name,
        fn_name,
    );

    ReturnType::Type(Default::default(), Box::new(modified_return_type))
}

// utilize `visit_type_mut` to handle all the variants of the return type in `syn`
// otherwise, we would have to write a lot of match arms
fn visit_type(ty: &mut Type, visitor: impl Fn(&mut TypePath)) {
    struct TypeVisitor<F>(F);
    impl<F: Fn(&mut TypePath)> VisitMut for TypeVisitor<F> {
        fn visit_type_path_mut(&mut self, type_path: &mut TypePath) {
            (self.0)(type_path);
        }
    }
    TypeVisitor(visitor).visit_type_mut(ty);
}

fn recursively_modify_return_type(
    ty: &mut Type,
    generic_idents: Vec<syn::GenericArgument>,
    struct_name: &Ident,
    fn_name: &Ident,
) {
    visit_type(ty, |type_path| {
        // Check each segment in the path
        for segment in type_path.path.segments.iter_mut() {
            if segment.ident == *struct_name {
                modify_segment(segment, generic_idents.clone(), fn_name);
            }
        }
    });
}

fn modify_segment(
    segment: &mut syn::PathSegment,
    generic_idents: Vec<syn::GenericArgument>,
    fn_name: &Ident,
) {
    match &mut segment.arguments {
        PathArguments::AngleBracketed(arguments) => {
            arguments.args.extend(generic_idents);
        }
        PathArguments::None => {
            segment.arguments =
                PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args: generic_idents.into_iter().collect(),
                    colon2_token: None,
                    lt_token: Default::default(),
                    gt_token: Default::default(),
                });
        }
        _ => panic!(
            "Function `{}`: Unsupported arguments in return type of the function.",
            fn_name
        ),
    }
}
