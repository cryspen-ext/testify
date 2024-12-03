use std::collections::HashMap;

// NOTE: pour les valeurs complexes, en fait le soucis principal,
// c'est les références. Les bindings mutables, c'est simple. Mais les
// refs (mutables ou non), c'est pénible. Dans le cas des
// préconditions, on peut juste : définir une statique mutable
// `Option<T>` par input (ou alias), et les set de manière
// unsafe. Quand on a terminé, on set à `None`. Comme ça on a des
// références (mutables si besoin) statiques partout !

/// A complex input value is a input instantiation which needs a setup
/// phase before use. A complex input value is always used via a
/// binding. The name of the binding is determined by the name input.
#[derive(Clone)]
pub struct ComplexInputValue {
    /// A sequence of bindings that will be inserted in the root
    /// expression of the test function. The resources introduced with
    /// those bindings have the lifetime of the test block.
    pub bindings: Vec<(syn::Pat, syn::Expr)>,
    /// Will the binding introduced be mutable?
    pub mutable: bool,
    /// What is the right-hand side of the binding?
    pub rhs: syn::Expr,
}

pub type Substs = HashMap<syn::Ident, syn::Ident>;

// pub trait RefreshNames {
//     fn refresh_names(&mut self, already_used_names: &AlreadyUsedNames);
// }

// impl RefreshNames for syn::Ident {
//     fn refresh_names(&mut self, already_used_names: &AlreadyUsedNames) {
//         if already_used_names.contains(self) {
//             let base = &format!("{}_", self);
//             let suffix = already_used_names
//                 .iter()
//                 .filter_map(|ident| ident.to_string().strip_prefix(base).map(|s| s.to_string()))
//                 .filter_map(|s| str::parse::<u32>(&s).ok())
//                 .max()
//                 .unwrap_or(0)
//                 + 1;

//             *self = syn::Ident::new(&format!("{base}{suffix}"), self.span());
//         }
//     }
// }

// impl RefreshNames for ComplexInputValue {
//     fn refresh_names(&mut self, already_used_names: &AlreadyUsedNames) {
//         let refresh_pat = |pat: &mut syn::Pat| {
//             struct Visitor {
//                 pattern_names: AlreadyUsedNames,
//                 already_used_names: AlreadyUsedNames,
//                 replaced: HashMap<syn::Ident, syn::Ident>,
//             }
//             use syn::visit_mut::*;
//             impl VisitMut for Visitor {
//                 fn visit_pat_ident_mut(&mut self, i: &mut syn::PatIdent) {
//                     let original = i.ident.clone();
//                     let names = (&self.pattern_names)
//                         .iter()
//                         .filter(|ident| *ident != &original)
//                         .chain((&self.already_used_names).iter())
//                         .cloned()
//                         .collect();
//                     i.ident.refresh_names(&names);
//                     if i.ident != original {
//                         self.pattern_names.insert(i.ident.clone());
//                         self.replaced.insert(original, i.ident.clone());
//                     }
//                     visit_pat_ident_mut(self, i);
//                 }
//             }
//             use syn::visit::Visit;
//             use syn::visit_mut::VisitMut;
//             let mut visitor = Visitor {
//                 pattern_names: {
//                     let mut visitor = visitors::IdentCollector::default();
//                     visitor.visit_pat(pat);
//                     visitor.idents()
//                 },
//                 already_used_names: already_used_names.clone(),
//                 replaced: HashMap::new(),
//             };
//             visitor.visit_pat_mut(pat);
//             visitor.replaced
//         };

//         fn bulk_replace_idents(expr: &mut syn::Expr, substs: &HashMap<syn::Ident, syn::Ident>) {
//             use syn::visit_mut::VisitMut;
//             /// Substitutes idents in bulk, assuming no shadowing occurs
//             struct BulkIdentSubst<'a> {
//                 substs: &'a HashMap<syn::Ident, syn::Ident>,
//             }
//             impl<'a> VisitMut for BulkIdentSubst<'a> {
//                 fn visit_ident_mut(&mut self, i: &mut syn::Ident) {
//                     if let Some(replacement) = self.substs.get(i) {
//                         *i = replacement.clone();
//                     }
//                 }
//             }
//             BulkIdentSubst { substs }.visit_expr_mut(expr);
//         }

//         let mut substs = HashMap::new();
//         for (pat, expr) in self.bindings.iter_mut() {
//             substs.extend(refresh_pat(pat).into_iter());
//             bulk_replace_idents(expr, &substs);
//         }
//         bulk_replace_idents(&mut self.rhs, &substs);
//     }
// }
