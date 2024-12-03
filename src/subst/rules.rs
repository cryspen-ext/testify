use crate::prelude::*;

struct Rewrite;
use crate::subst::syn_utils::NAryCall;
use syn::visit_mut::*;
use syn::{Expr, ExprBlock, ExprIf, ExprLit, Lit};

// /// We want to rewrite only nodes marked as rewrite
// fn expect_rewrite_node

impl VisitMut for Rewrite {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        visit_expr_mut(self, expr);
        match expr {
            Expr::If(ExprIf {
                cond,
                then_branch,
                else_branch,
                ..
            }) => {
                let Ok(call) = NAryCall::<1>::try_from(&**cond) else {
                    return;
                };
                let [Expr::Lit(ExprLit {
                    lit: Lit::Bool(bool_value),
                    ..
                })] = &call.args
                else {
                    return;
                };
                if bool_value.value {
                    *expr = Expr::Block(ExprBlock {
                        attrs: vec![],
                        label: None,
                        block: then_branch.clone(),
                    });
                } else {
                    *expr = else_branch
                        .as_ref()
                        .map(|(_, expr)| expr.as_ref().clone())
                        .unwrap_or(parse_quote! {()});
                }
            }
            _ => (),
        }
    }
}
