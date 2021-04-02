mod sig;
mod utils;
mod walker;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn};

use sig::extract_iter_sig;
use utils::{make_ident, to_pascal_case};
use walker::Walker;

#[proc_macro_attribute]
pub fn giver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let iter_item_type;
    let iter_params;
    match extract_iter_sig(&func.sig) {
        Ok((params, item_type)) => {
            iter_item_type = item_type;
            iter_params = params;
        }
        Err((span, message)) => {
            return fail(&func, span, message);
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

    let struct_params = iter_params.iter().map(|(ident, typ)| {
        let field_ident = make_ident(&format!("param_{}", ident.to_string()));
        quote! { #field_ident: #typ }
    });

    let new_params = iter_params.iter().map(|(ident, typ)| {
        quote! { #ident: #typ }
    });

    let params_assign = iter_params.iter().map(|(ident, _typ)| {
        let field_ident = make_ident(&format!("param_{}", ident.to_string()));
        quote! { #field_ident: #ident }
    });

    let new_code = quote! {
        mod #mod_name {
            enum #state_enum_name { #(#state_idents),* }

            struct #struct_name {
                state: #state_enum_name,
                #(#struct_params),*
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

            pub fn #func_name(#(#new_params),*) -> impl Iterator<Item = #iter_item_type> {
                #struct_name {
                    state: #state_enum_name::S0_Start,
                    #(#params_assign),*
                }
            }
        }

        #func_vis use #mod_name::#func_name;
    };

    if attr.to_string() == "print" {
        println!("{}", &new_code);
    }

    return TokenStream::from(new_code);
}

fn fail<T: Spanned + ?Sized>(func: &ItemFn, s: &T, msg: &str) -> TokenStream {
    let msg = format!("[be_generust] {}", msg);
    let err = syn::Error::new(s.span(), msg).to_compile_error();

    let dummy_vis = &func.vis;
    let dummy_sig = &func.sig;
    // We can't just use `unimplemented!()` due to the ff bug:
    // https://github.com/rust-lang/rust/issues/36375
    let dummy = quote! {
        #dummy_vis #dummy_sig { std::iter::empty() }
    };

    return quote! {
        #err
        #dummy
    }
    .into();
}
