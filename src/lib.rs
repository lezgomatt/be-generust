use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Expr, ItemFn, Stmt};

struct Walker {
    name: String,
    states: Vec<String>,
    output: BTreeMap<(usize, String), Vec<Stmt>>,
}

fn new_walker(name: String) -> Walker {
    let start_state = "S0_Start".to_string();

    let mut states = Vec::new();
    states.push(start_state.clone());

    let mut output = BTreeMap::new();
    output.insert((0, start_state), Vec::new());

    Walker {
        name,
        states,
        output,
    }
}

fn walk_fn_body(w: &mut Walker, body: &Vec<Stmt>) {
    for s in body {
        match s {
            Stmt::Semi(e, _) => match e {
                Expr::Macro(mac_expr) => {
                    if !mac_expr.mac.path.is_ident("give") {
                        panic!("Unsupported");
                    }

                    let curr_state = w.states.last().unwrap().clone();
                    let num_states = w.states.len();
                    let next_state = format!("S{}_{}", num_states, "AFTER_GIVE");
                    w.states.push(next_state.clone());
                    w.output
                        .insert((num_states, next_state.clone()), Vec::new());

                    let state_enum = make_ident(&w.name);
                    let state_id = make_ident(&next_state);
                    let give_expr = &mac_expr.mac.tokens;

                    let assign: Stmt = parse_quote! { self.state = #state_enum::#state_id; };
                    let ret: Stmt = parse_quote! { return Some(#give_expr); };

                    let block = w
                        .output
                        .get_mut(&(num_states - 1, curr_state.clone()))
                        .unwrap();
                    block.push(assign);
                    block.push(ret);
                }
                _ => panic!("Unsupported"),
            },
            Stmt::Local(_) | Stmt::Item(_) | Stmt::Expr(_) => panic!("Unsupported"),
        }
    }

    let num_states = w.states.len();
    let next_state = format!("S{}_End", num_states);
    w.states.push(next_state.clone());
    w.output
        .insert((num_states, next_state.clone()), Vec::new());

    let state_enum = make_ident(&w.name);
    let state_id = make_ident(&next_state);

    let ret: Stmt = parse_quote! { return None; };
    let next_block = w.output.get_mut(&(num_states, next_state.clone())).unwrap();
    next_block.push(ret);
}

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

    let mut w = new_walker(format!("{}State", name_pascal));
    walk_fn_body(&mut w, &func.block.stmts);
    let state_idents = w.states.iter().map(|s| make_ident(&s));
    let match_blocks = w.output.iter().map(|((_, s), b)| {
        let state_enum = make_ident(&w.name);
        let state_id = make_ident(&s);

        if b.is_empty() {
            quote! {
                #state_enum::#state_id |
            }
        } else {
            quote! {
                #state_enum::#state_id => {
                    #(#b)*
                },
            }
        }
    });

    let new_code = quote! {
        mod #mod_name {
            enum #state_enum_name { #(#state_idents),* }

            struct #struct_name {
                state: #state_enum_name,
            }

            impl Iterator for #struct_name {
                type Item = #iter_item_type;

                fn next(&mut self) -> Option<#iter_item_type> {
                    loop {
                        match self.state {
                            #(#match_blocks)*
                        }
                    }
                }
            }

            pub fn #func_name() -> impl Iterator<Item = #iter_item_type> {
                #struct_name { state: #state_enum_name::S0_Start }
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
