use proc_macro2::Ident;
use std::collections::BTreeMap;
use syn::{parse_quote, Expr, Stmt};

use crate::utils::make_ident;

type StateKey = (usize, Ident);

pub(crate) struct Walker {
    pub name: Ident,
    pub states: Vec<Ident>,
    pub output: BTreeMap<StateKey, Vec<Stmt>>,
    pub params: Vec<Ident>,
}

impl Walker {
    pub fn walk(name: Ident, body: &Vec<Stmt>) -> Walker {
        let mut w = Walker {
            name,
            states: Vec::new(),
            output: BTreeMap::new(),
            params: Vec::new(),
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

    fn clone_stmt(&self, stmt: &Stmt) -> Stmt {
        let mut result = stmt.clone();

        match stmt {
            Stmt::Semi(e, semi) => {
                result = Stmt::Semi(self.clone_expr(&e), semi.clone());
            }
            Stmt::Expr(e) => {
                result = Stmt::Expr(self.clone_expr(&e));
            }
            Stmt::Local(l) => {
                let mut local = l.clone();
                match local.init {
                    None => {}
                    Some((eq, box_expr)) => {
                        let new_expr = Box::new(self.clone_expr(&*box_expr));
                        local.init = Some((eq.clone(), new_expr));
                    }
                }
            }
            Stmt::Item(_) => {}
        }

        return result;
    }

    fn clone_expr(&self, expr: &Expr) -> Expr {
        let mut result = expr.clone();

        // match expr {
        //     Expr::Array(e_array) => {}
        //     Expr::Assign(e_assign) => {}
        //     Expr::AssignOp(e_assign_op) => {}
        //     Expr::Async(e_async) => {}
        //     Expr::Await(e_await) => {}
        //     Expr::Binary(e_binary) => {}
        //     Expr::Block(e_block) => {}
        //     Expr::Box(e_box) => {}
        //     Expr::Break(e_break) => {}
        //     Expr::Call(e_call) => {}
        //     Expr::Cast(e_cast) => {}
        //     Expr::Closure(e_closure) => {}
        //     Expr::Continue(e_continue) => {}
        //     Expr::Field(e_field) => {}
        //     Expr::ForLoop(e_for_loop) => {}
        //     Expr::Group(e_group) => {}
        //     Expr::If(e_if) => {}
        //     Expr::Index(e_index) => {}
        //     Expr::Let(e_let) => {}
        //     Expr::Lit(e_lit) => {}
        //     Expr::Loop(e_loop) => {}
        //     Expr::Macro(e_macro) => {}
        //     Expr::Match(e_match) => {}
        //     Expr::MethodCall(e_method_call) => {}
        //     Expr::Paren(e_paren) => {}
        //     Expr::Path(e_path) => {}
        //     Expr::Range(e_range) => {}
        //     Expr::Reference(e_reference) => {}
        //     Expr::Repeat(e_repeat) => {}
        //     Expr::Return(e_return) => {}
        //     Expr::Struct(e_struct) => {}
        //     Expr::Try(e_try) => {}
        //     Expr::TryBlock(e_try_block) => {}
        //     Expr::Tuple(e_tuple) => {}
        //     Expr::Type(e_type) => {}
        //     Expr::Unary(e_unary) => {}
        //     Expr::Unsafe(e_unsafe) => {}
        //     Expr::Verbatim(e_verbatim) => {}
        //     Expr::While(e_while) => {}
        //     Expr::Yield(e_yield) => {}
        // }

        return result;
    }

    fn copy_stmt(&mut self, stmt: &Stmt) {
        self.add_stmt(&self.curr_state(), self.clone_stmt(stmt));
    }

    fn curr_state(&self) -> StateKey {
        return (self.states.len() - 1, self.states.last().unwrap().clone());
    }

    fn add_stmt(&mut self, state: &StateKey, stmt: Stmt) {
        self.output.get_mut(state).unwrap().push(stmt);
    }
}
