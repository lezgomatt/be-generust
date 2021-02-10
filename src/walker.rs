use proc_macro2::Ident;
use std::collections::BTreeMap;
use syn::{parse_quote, Expr, Stmt};

use crate::utils::make_ident;

type StateKey = (usize, Ident);

pub(crate) struct Walker {
    pub name: Ident,
    pub states: Vec<Ident>,
    pub output: BTreeMap<StateKey, Vec<Stmt>>,
}

impl Walker {
    pub fn walk(name: Ident, body: &Vec<Stmt>) -> Walker {
        let mut w = Walker {
            name,
            states: Vec::new(),
            output: BTreeMap::new(),
        };

        w.walk_fn_body(body);

        return w;
    }

    fn add_state(&mut self, name: &str) -> StateKey {
        let state_num = self.states.len();
        let state_label = make_ident(&format!("S{}_{}", state_num, name));
        let new_state = (state_num, state_label);

        self.states.push(new_state.1.clone());
        self.output.insert(new_state.clone(), Vec::new());

        return new_state;
    }

    fn walk_fn_body(&mut self, body: &Vec<Stmt>) {
        self.add_state("Start");

        for s in body {
            match s {
                Stmt::Semi(e, _) => match e {
                    Expr::Macro(mac_expr) => {
                        if !mac_expr.mac.path.is_ident("give") {
                            self.copy_stmt(s);
                            continue;
                        }

                        let curr_state = self.curr_state();
                        let next_state = self.add_state("AfterGive");

                        self.add_stmt(&curr_state, {
                            let enom = &self.name;
                            let label = &next_state.1;
                            parse_quote! { self.state = #enom::#label; }
                        });

                        self.add_stmt(&curr_state, {
                            let give_expr = &mac_expr.mac.tokens;
                            parse_quote! { return Some(#give_expr); }
                        });
                    }
                    _ => {
                        self.copy_stmt(s);
                    }
                },
                Stmt::Local(_) | Stmt::Item(_) | Stmt::Expr(_) => {
                    self.copy_stmt(s);
                }
            }
        }

        let end_state = self.add_state("End");
        self.add_stmt(&end_state, parse_quote! { return None; });
    }

    fn copy_stmt(&mut self, stmt: &Stmt) {
        self.add_stmt(&self.curr_state(), stmt.clone());
    }

    fn curr_state(&self) -> StateKey {
        return (self.states.len() - 1, self.states.last().unwrap().clone());
    }

    fn add_stmt(&mut self, state: &StateKey, stmt: Stmt) {
        self.output.get_mut(state).unwrap().push(stmt);
    }
}
