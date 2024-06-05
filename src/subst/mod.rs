use syn::visit::Visit;
use syn::visit_mut::*;

mod syn_utils;
pub(crate) mod visitors;

use syn_utils::*;
use visitors::*;

pub trait Subst<Subject>: Clone {
    fn subst(subject: &mut Subject, binding: &str, replacement: Self);
}

pub trait SubstHelper<Replacement> {
    fn subst(&mut self, binding: &str, replacement: Replacement);
}

impl<Subject, Replacement: Subst<Subject>> SubstHelper<Replacement> for Subject {
    fn subst(&mut self, binding: &str, replacement: Replacement) {
        Replacement::subst(self, binding, replacement)
    }
}

macro_rules! derive_subst {
    ($typ:ty, $meth:ident) => {
        impl Subst<$typ> for syn::Expr {
            fn subst(subject: &mut $typ, binding: &str, replacement: Self) {
                let mut visitor = CollectedIntroducedVariables::new((binding, &replacement));
                visitor.$meth(subject);
            }
        }

        impl Subst<$typ> for syn::Type {
            fn subst(subject: &mut $typ, binding: &str, replacement: Self) {
                let mut visitor = CollectedIntroducedVariables::new((binding, &replacement));
                visitor.$meth(subject);
            }
        }
    };
}

derive_subst!(syn::Type, visit_type_mut);
derive_subst!(syn::Expr, visit_expr_mut);
derive_subst!(syn::WhereClause, visit_where_clause_mut);

pub trait ETSubst: Subst<syn::Type> + Subst<syn::Expr> + Subst<syn::WhereClause> {}

impl<T: Subst<syn::Type> + Subst<syn::Expr> + Subst<syn::WhereClause>> ETSubst for T {}
