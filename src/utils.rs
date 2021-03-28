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
