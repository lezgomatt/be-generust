use std::collections::BTreeMap;
use syn::{parse_quote, Expr, Stmt};

use crate::utils::make_ident;

pub(crate) struct Walker {
    pub name: String,
    pub states: Vec<String>,
    pub output: BTreeMap<(usize, String), Vec<Stmt>>,
}

pub(crate) fn new_walker(name: String) -> Walker {
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

pub(crate) fn walk_fn_body(w: &mut Walker, body: &Vec<Stmt>) {
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
