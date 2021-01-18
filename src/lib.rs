use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn giver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let str_name_snake = func.sig.ident.to_string();
    let str_name_pascal = to_pascal_case(&str_name_snake);

    let func_name = make_ident(&str_name_snake);
    let mod_name = make_ident(&(str_name_snake.clone() + "_mod"));
    let state_enum_name = make_ident(&(str_name_pascal.clone() + "State"));
    let struct_name = make_ident(&str_name_pascal);

    let new_code = quote! {
        mod #mod_name {
            enum #state_enum_name { Start, Done }

            pub struct #struct_name {
                state: #state_enum_name,
            }

            impl Iterator for #struct_name {
                type Item = i64;

                fn next(&mut self) -> Option<i64> {
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

fn make_ident(str: &str) -> Ident {
    Ident::new(str, Span::call_site())
}
