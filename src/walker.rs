use std::collections::BTreeMap;
use syn::{parse_quote, Expr, Stmt};

use crate::utils::make_ident;

type StateKey = (usize, String);

pub(crate) struct Walker {
    pub name: String,
    pub states: Vec<String>,
    pub output: BTreeMap<StateKey, Vec<Stmt>>,
}

impl Walker {
    pub fn walk(name: String, body: &Vec<Stmt>) -> Walker {
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
        let new_state = (state_num, format!("S{}_{}", state_num, name));

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
                            self.add_stmt(&self.curr_state(), s.clone());
                        } else {
                            let curr_state = self.curr_state();
                            let next_state = self.add_state("AfterGive");

                            let state_enum = make_ident(&self.name);
                            let state_id = make_ident(&next_state.1);
                            let give_expr = &mac_expr.mac.tokens;

                            let assign: Stmt =
                                parse_quote! { self.state = #state_enum::#state_id; };
                            let ret: Stmt = parse_quote! { return Some(#give_expr); };

                            self.add_stmt(&curr_state, assign);
                            self.add_stmt(&curr_state, ret);
                        }
                    }
                    _ => {
                        self.add_stmt(&self.curr_state(), s.clone());
                    }
                },
                Stmt::Local(_) | Stmt::Item(_) | Stmt::Expr(_) => {
                    self.add_stmt(&self.curr_state(), s.clone());
                }
            }
        }

        let end_state = self.add_state("End");
        let ret: Stmt = parse_quote! { return None; };
        self.add_stmt(&end_state, ret);
    }

    fn curr_state(&self) -> StateKey {
        return (self.states.len() - 1, self.states.last().unwrap().clone());
    }

    fn add_stmt(&mut self, state: &StateKey, stmt: Stmt) {
        self.output.get_mut(state).unwrap().push(stmt);
    }
}
