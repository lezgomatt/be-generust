use std::collections::BTreeMap;
use syn::{parse_quote, Expr, Stmt};

use crate::utils::make_ident;

pub(crate) struct Walker {
    pub name: String,
    pub states: Vec<String>,
    pub output: BTreeMap<(usize, String), Vec<Stmt>>,
}

impl Walker {
    pub fn new(name: String) -> Walker {
        let mut w = Walker {
            name,
            states: Vec::new(),
            output: BTreeMap::new(),
        };

        w.add_state("Start");

        return w;
    }

    pub fn add_state(&mut self, name: &str) -> (usize, String) {
        let num_states = self.states.len();

        let new_state = format!("S{}_{}", num_states, name);

        self.states.push(new_state.clone());
        self.output
            .insert((num_states, new_state.clone()), Vec::new());

        return (num_states, new_state);
    }

    pub fn walk_fn_body(&mut self, body: &Vec<Stmt>) {
        for s in body {
            match s {
                Stmt::Semi(e, _) => match e {
                    Expr::Macro(mac_expr) => {
                        if !mac_expr.mac.path.is_ident("give") {
                            panic!("Unsupported");
                        }

                        let curr_state = self.states.last().unwrap().clone();
                        let (num_states, next_state) = self.add_state("AfterGive");

                        let state_enum = make_ident(&self.name);
                        let state_id = make_ident(&next_state);
                        let give_expr = &mac_expr.mac.tokens;

                        let assign: Stmt = parse_quote! { self.state = #state_enum::#state_id; };
                        let ret: Stmt = parse_quote! { return Some(#give_expr); };

                        let block = self
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

        let (num_states, next_state) = self.add_state("End");

        let ret: Stmt = parse_quote! { return None; };
        let next_block = self
            .output
            .get_mut(&(num_states, next_state.clone()))
            .unwrap();
        next_block.push(ret);
    }
}
