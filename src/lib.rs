use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn giver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let iter_item_type = match get_iter_item_type(&func.sig.output) {
        Some(ty) => ty,
        None => {
            return fail(
                &func.sig.output,
                "return type must be `-> impl Iterator<Item = XXX>`",
            )
        }
    };

    let name_snake = func.sig.ident.to_string();
    let name_pascal = to_pascal_case(&name_snake);

    let func_name = make_ident(&name_snake);
    let mod_name = make_ident(&format!("{}_mod", name_snake));
    let state_enum_name = make_ident(&format!("{}State", name_pascal));
    let struct_name = make_ident(&name_pascal);

    let new_code = quote! {
        mod #mod_name {
            enum #state_enum_name { Start, Done }

            pub struct #struct_name {
                state: #state_enum_name,
            }

            impl Iterator for #struct_name {
                type Item = #iter_item_type;

                fn next(&mut self) -> Option<#iter_item_type> {
                    loop {
                        match self.state {
                            #state_enum_name::Start => {
                                self.state = #state_enum_name::Done;
                                return Some(1);
                            },
                            #state_enum_name::Done => {
                                return None
                            },
                        }
                    }
                }
            }

            pub fn #func_name() -> #struct_name {
                #struct_name { state: #state_enum_name::Start }
            }
        }

        use #mod_name::#func_name;
    };

    if attr.to_string() == "print" {
        println!("{}", &new_code);
    }

    TokenStream::from(new_code)
}

fn to_pascal_case(snake_case_str: &str) -> String {
    let mut result = String::new();

    for chunk in snake_case_str.split("_") {
        if let Some(c) = chunk.chars().nth(0) {
            result += &c.to_uppercase().to_string();
            result += &chunk[1..];
        }
    }

    return result;
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

fn get_iter_item_type(ret_type: &syn::ReturnType) -> Option<&syn::Type> {
    let boxed_type = must_match!(ret_type, syn::ReturnType::Type(_, bt), bt);

    // impl Iterator<Item = XXX>
    let impl_bound = must_match!(
        &**boxed_type,
        syn::Type::ImplTrait(it),
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
        &trait_segment.arguments,
        syn::PathArguments::AngleBracketed(ga),
        sole_elem!(ga.args)
    );

    // Item = XXX
    let binding = must_match!(&generic_arg, syn::GenericArgument::Binding(b), b);
    if binding.ident.to_string() != "Item" {
        return None;
    }

    Some(&binding.ty)
}

fn make_ident(str: &str) -> Ident {
    Ident::new(str, Span::call_site())
}

fn fail<T: Spanned>(s: &T, msg: &str) -> TokenStream {
    let msg = format!("[generoust] {}", msg);
    let err = syn::Error::new(s.span(), msg).to_compile_error();

    TokenStream::from(err)
}
