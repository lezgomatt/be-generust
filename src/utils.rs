pub(crate) fn to_pascal_case(snake_case_str: &str) -> String {
    let mut result = String::new();

    for chunk in snake_case_str.split("_") {
        if let Some(c) = chunk.chars().nth(0) {
            result += &c.to_uppercase().to_string();
            result += &chunk[1..];
        }
    }

    return result;
}

pub(crate) fn make_ident(str: &str) -> proc_macro2::Ident {
    use proc_macro2::{Ident, Span};

    return Ident::new(str, Span::call_site());
}

macro_rules! must_match {
    ($e:expr, $p:pat, $r:expr) => {
        match $e {
            $p => $r,
            _ => return None,
        }
    };
}

macro_rules! sole_elem {
    ($e:expr) => {
        if $e.len() == 1 {
            &$e[0]
        } else {
            return None;
        }
    };
}

pub(crate) fn get_iter_item_type(ret_type: &syn::ReturnType) -> Option<&syn::Type> {
    let boxed_type = must_match!(ret_type, syn::ReturnType::Type(_, bt), bt);

    // impl Iterator<Item = XXX>
    let impl_bound = must_match!(
        **boxed_type,
        syn::Type::ImplTrait(ref it),
        sole_elem!(it.bounds)
    );

    // Iterator<Item = XXX>
    let trait_segment = must_match!(
        impl_bound,
        syn::TypeParamBound::Trait(tb),
        sole_elem!(tb.path.segments)
    );
    if trait_segment.ident.to_string() != "Iterator" {
        return None;
    }

    // <Item = XXX>
    let generic_arg = must_match!(
        trait_segment.arguments,
        syn::PathArguments::AngleBracketed(ref ga),
        sole_elem!(ga.args)
    );

    // Item = XXX
    let binding = must_match!(generic_arg, syn::GenericArgument::Binding(ref b), b);
    if binding.ident.to_string() != "Item" {
        return None;
    }

    return Some(&binding.ty);
}
