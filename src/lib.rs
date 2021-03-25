mod utils;
mod walker;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn, Signature};

use utils::{get_iter_item_type, make_ident, to_pascal_case};
use walker::Walker;

#[proc_macro_attribute]
pub fn giver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let iter_item_type;
    match check_sig(&func.sig) {
        Ok((_params, item_type)) => {
            iter_item_type = item_type;
        }
        Err(failure) => {
            return failure;
        }
    }

    let func_vis = &func.vis;

    let name_snake = func.sig.ident.to_string();
    let name_pascal = to_pascal_case(&name_snake);

    let func_name = make_ident(&name_snake);
    let mod_name = make_ident(&format!("{}_mod", name_snake));
    let state_enum_name = make_ident(&format!("{}State", name_pascal));
    let struct_name = make_ident(&name_pascal);

    let w = Walker::walk(state_enum_name.clone(), &func.block.stmts);
    let state_idents = &w.states;
    let match_blocks = w.output.iter().map(|((_, s), b)| {
        let state_enum = &w.name;

        if b.is_empty() {
            quote! {
                #state_enum::#s |
            }
        } else {
            quote! {
                #state_enum::#s => {
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

        #func_vis use #mod_name::#func_name;
    };

    if attr.to_string() == "print" {
        println!("{}", &new_code);
    }

    return TokenStream::from(new_code);
}

fn fail<T: Spanned>(s: &T, msg: &str) -> TokenStream {
    let msg = format!("[be_generust] {}", msg);
    let err = syn::Error::new(s.span(), msg).to_compile_error();

    return TokenStream::from(err);
}

fn check_sig(sig: &Signature) -> Result<(Vec::<(proc_macro2::Ident, syn::Type)>, syn::Type), TokenStream> {
    if sig.constness.is_some() {
        return Err(fail(&sig.constness, "iterator cannot be const"));
    }

    if sig.asyncness.is_some() {
        return Err(fail(&sig.asyncness, "iterator cannot be async"));
    }

    if sig.unsafety.is_some() {
        return Err(fail(&sig.unsafety, "iterator cannot be unsafe"));
    }

    if sig.abi.is_some() {
        return Err(fail(&sig.abi, "iterator cannot be extern"));
    }

    if sig.variadic.is_some() {
        return Err(fail(
            &sig.variadic,
            "iterator cannot have variadic parameters",
        ));
    }

    if let Some(arg) = sig.inputs.first() {
        if let syn::FnArg::Receiver(_) = arg {
            return Err(fail(
                &arg,
                "iterator cannot have a method receiver (self)",
            ));
        }
    }

    let mut params = Vec::<(proc_macro2::Ident, syn::Type)>::new();

    for arg in sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => {
                    if pat_ident.by_ref.is_some() {
                        return Err(fail(
                            &pat_ident.by_ref,
                            "iterator cannot have reference arguments",
                        ));
                    }

                    if let Some((_, ref subpat)) = pat_ident.subpat {
                        return Err(fail(
                            subpat,
                            "iterator cannot have argument subpatterns",
                        ));
                    }

                    params.push((pat_ident.ident.clone(), (*pat_type.ty).clone()));
                }
                _ => {
                    return Err(fail(
                        &arg,
                        "iterator cannot have a pattern arguments",
                    ));
                }
            }
        }
    }

    match get_iter_item_type(&sig.output) {
        Some(ty) => Ok((params, ty.clone())),
        None => {
            return Err(fail(
                &sig.output,
                "return type must be `-> impl Iterator<Item = XXX>`",
            ));
        }
    }
}
