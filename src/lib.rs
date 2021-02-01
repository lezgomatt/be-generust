mod utils;
mod walker;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn};

use utils::{get_iter_item_type, make_ident, to_pascal_case};
use walker::Walker;

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

        use #mod_name::#func_name;
    };

    if attr.to_string() == "print" {
        println!("{}", &new_code);
    }

    TokenStream::from(new_code)
}

fn fail<T: Spanned>(s: &T, msg: &str) -> TokenStream {
    let msg = format!("[generoust] {}", msg);
    let err = syn::Error::new(s.span(), msg).to_compile_error();

    TokenStream::from(err)
}
