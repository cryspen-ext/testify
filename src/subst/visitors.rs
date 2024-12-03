use super::syn_utils::*;
use crate::prelude::*;
use std::collections::HashSet;
use syn::parse_quote;
use syn::visit::*;
use syn::visit_mut::*;

pub type SubstReferences = HashSet<u32>;
pub type AlreadyUsedNames = HashSet<syn::Ident>;

pub struct SubstVisitor<Value> {
    pub already_used_names: AlreadyUsedNames,
    pub binding: String,
    pub value: Box<dyn FnMut(&AlreadyUsedNames, u32) -> Value>,
    pub substitued: SubstReferences,
}

impl<Value> SubstVisitor<Value> {
    pub fn new(
        binding: &str,
        already_used_names: AlreadyUsedNames,
        value: impl Fn(&AlreadyUsedNames, u32) -> Value + 'static,
    ) -> Self {
        Self {
            already_used_names,
            binding: binding.to_string(),
            value: Box::new(value),
            substitued: HashSet::new(),
        }
    }
    pub fn value(&mut self, nth: u32) -> Value {
        self.substitued.insert(nth);
        (self.value)(&self.already_used_names, 0)
    }
}

/// To refer a value input `x` from a contract, one should write
/// `x(n)`, with `n` the `n`th instantiation of `x`.
fn expect_input_reference(input_name: &str, expr: &syn::Expr) -> Option<u32> {
    let syn::Expr::Call(call) = expr else {
        return None;
    };
    call.func
        .expect_ident()
        .filter(|ident| &ident.to_string() == input_name)?;
    let args = call.args.to_vec();
    match args.as_slice() {
        [syn::Expr::Lit(lit)] => {
            if let syn::Lit::Int(nth) = &lit.lit {
                Some(nth.base10_digits().parse().ok()?)
            } else {
                None
            }
        }
        // `x()` is a shortcut for `x(0)`
        [] => Some(0),
        _ => None,
    }
}

impl<T> SubstVisitor<T> {
    fn pattern_contains_binding(&self, pat: &syn::Pat) -> bool {
        let mut visitor = IdentCollector::default();
        visitor.visit_pat(&pat);
        visitor
            .idents()
            .iter()
            .any(|i| i.to_string() == self.binding)
    }
}

impl VisitMut for SubstVisitor<syn::Type> {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        if let syn::Type::Path(type_path) = ty {
            if let Some(ident) = type_path.path.expect_ident() {
                if ident.to_string() == self.binding {
                    return *ty = self.value(0);
                }
            }
        }
        visit_type_mut(self, ty)
    }
    fn visit_item_mut(&mut self, _fn_item: &mut syn::Item) {
        panic!("Nested items are not supported yet")
    }
}

impl VisitMut for SubstVisitor<syn::Expr> {
    fn visit_expr_let_mut(&mut self, let_expr: &mut syn::ExprLet) {
        if !self.pattern_contains_binding(&*let_expr.pat) {
            visit_expr_let_mut(self, let_expr)
        }
    }
    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        if !closure
            .inputs
            .iter()
            .any(|pat| self.pattern_contains_binding(pat))
        {
            visit_expr_closure_mut(self, closure);
        }
    }
    fn visit_item_mut(&mut self, _fn_item: &mut syn::Item) {
        panic!("Nested items are not supported yet")
    }
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if expr
            .expect_ident()
            .filter(|ident| &ident.to_string() == &self.binding)
            .is_some()
        {
            return *expr = self.value(0);
        }
        match expect_input_reference(&self.binding, expr) {
            Some(nth) => *expr = self.value(nth),
            _ => visit_expr_mut(self, expr),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdentCollector {
    idents: HashSet<syn::Ident>,
}

impl IdentCollector {
    pub fn idents(self) -> HashSet<syn::Ident> {
        self.idents
    }
}

impl<'a> Visit<'a> for IdentCollector {
    fn visit_path(&mut self, path: &'a syn::Path) {
        if let Some(ident) = path.expect_ident() {
            self.idents.insert(ident);
        };
        // We need to recurse under paths, e.g. `f::<x>` refers to
        // both `f` and `x`
        visit_path(self, path);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubstituerControl {
    Continue,
    Stop,
}

pub trait Substituer {
    fn subst_expr(
        &mut self,
        _context: &HashSet<syn::Ident>,
        _expr: &mut syn::Expr,
    ) -> SubstituerControl {
        SubstituerControl::Continue
    }
    fn subst_type(
        &mut self,
        _context: &HashSet<syn::Ident>,
        _typ: &mut syn::Type,
    ) -> SubstituerControl {
        SubstituerControl::Continue
    }
}

impl Substituer for () {}
impl<'a, 'b> Substituer for (&'a str, &'b syn::Expr) {
    fn subst_expr(
        &mut self,
        context: &HashSet<syn::Ident>,
        expr: &mut syn::Expr,
    ) -> SubstituerControl {
        if expr.is_ident(self.0) && !context.iter().any(|ident| &ident.to_string() == self.0) {
            *expr = self.1.clone();
            SubstituerControl::Stop
        } else {
            SubstituerControl::Continue
        }
    }
}
impl<'a, 'b> Substituer for (&'a str, &'b syn::Type) {
    fn subst_type(
        &mut self,
        context: &HashSet<syn::Ident>,
        typ: &mut syn::Type,
    ) -> SubstituerControl {
        if typ.is_ident(self.0) && !context.iter().any(|ident| &ident.to_string() == self.0) {
            *typ = self.1.clone();
            SubstituerControl::Stop
        } else {
            SubstituerControl::Continue
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    AddBound,
    AddFree,
}

#[derive(Debug, Clone)]
pub struct CollectedIntroducedVariables<S> {
    mode: Mode,
    bound_vars: HashSet<syn::Ident>,
    free_vars: HashSet<syn::Ident>,
    substituer: S,
}

impl<S: Substituer> CollectedIntroducedVariables<S> {
    pub fn new(substituer: S) -> Self {
        Self {
            mode: Mode::AddFree,
            bound_vars: HashSet::new(),
            free_vars: HashSet::new(),
            substituer,
        }
    }
    pub fn free_vars(&self) -> &HashSet<syn::Ident> {
        &self.free_vars
    }
}
impl<S> CollectedIntroducedVariables<S> {
    fn add_var(&mut self, var: syn::Ident) {
        match self.mode {
            Mode::AddBound => {
                self.bound_vars.insert(var);
            }
            Mode::AddFree if !self.bound_vars.contains(&var) => {
                self.free_vars.insert(var);
            }
            _ => (),
        }
    }
    fn with_mode(&mut self, mode: Mode, mut f: impl FnMut(&mut Self) -> ()) {
        let previous_mode = self.mode;
        self.mode = mode;
        f(self);
        self.mode = previous_mode;
    }
    fn with_bound_vars(&mut self, f: impl FnMut(&mut Self) -> ()) {
        self.with_mode(Mode::AddBound, f)
    }
    fn with_free_vars(&mut self, f: impl FnMut(&mut Self) -> ()) {
        self.with_mode(Mode::AddFree, f)
    }
    fn scope(&mut self, mut f: impl FnMut(&mut Self) -> ()) {
        let bound_vars = self.bound_vars.clone();
        f(self);
        self.bound_vars = bound_vars;
    }
}
impl<S: Substituer> CollectedIntroducedVariables<S> {
    fn visit_attributes_mut<'a>(&mut self, attrs: &'a mut Vec<syn::Attribute>) {
        for attr in attrs {
            self.visit_attribute_mut(attr)
        }
    }
}

impl<S: Substituer> VisitMut for CollectedIntroducedVariables<S> {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if self.substituer.subst_expr(&self.bound_vars, expr) == SubstituerControl::Continue {
            visit_expr_mut(self, expr)
        }
    }
    fn visit_type_mut(&mut self, typ: &mut syn::Type) {
        if self.substituer.subst_type(&self.bound_vars, typ) == SubstituerControl::Continue {
            visit_type_mut(self, typ)
        }
    }
    fn visit_ident_mut(&mut self, ident: &mut syn::Ident) {
        self.add_var(ident.clone());
    }
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Some(ident) = path.expect_ident() {
            self.add_var(ident.clone());
        };
        visit_path_mut(self, path);
    }
    fn visit_pat_mut(&mut self, pat: &mut syn::Pat) {
        use syn::Pat;
        match pat {
            Pat::Type(pat_type) => self.visit_pat_type_mut(pat_type),
            Pat::Const(_) | Pat::Range(_) => self.with_free_vars(|this| visit_pat_mut(this, pat)),
            _ => self.with_bound_vars(|this| visit_pat_mut(this, pat)),
        }
    }
    fn visit_pat_type_mut(&mut self, pat_type: &mut syn::PatType) {
        self.with_bound_vars(|this| this.visit_pat_mut(pat_type.pat.borrow_mut()));
        self.with_free_vars(|this| {
            this.visit_type_mut(&mut *pat_type.ty);
            this.visit_attributes_mut(&mut pat_type.attrs);
        })
    }
    fn visit_generic_param_mut(&mut self, generic_param: &mut syn::GenericParam) {
        match generic_param {
            syn::GenericParam::Type(tp) => {
                self.with_bound_vars(|this| this.visit_ident_mut(&mut tp.ident));
                self.with_free_vars(|this| {
                    this.visit_attributes_mut(&mut tp.attrs);
                    for bound in &mut tp.bounds {
                        this.visit_type_param_bound_mut(bound)
                    }
                    if let Some(ty) = &mut tp.default {
                        this.visit_type_mut(ty)
                    }
                })
            }
            syn::GenericParam::Const(cp) => {
                self.with_bound_vars(|this| this.visit_ident_mut(&mut cp.ident));
                self.with_free_vars(|this| {
                    this.visit_attributes_mut(&mut cp.attrs);
                    this.visit_type_mut(&mut cp.ty);
                    if let Some(ty) = &mut cp.default {
                        this.visit_expr_mut(ty)
                    }
                });
            }
            // we don't care about lifetime
            syn::GenericParam::Lifetime(_) => (),
        }
    }
    fn visit_local_mut(&mut self, i: &mut syn::Local) {
        self.visit_attributes_mut(&mut i.attrs);
        if let Some(init) = &mut i.init {
            self.visit_local_init_mut(init);
        }
        self.visit_pat_mut(&mut i.pat)
    }
    fn visit_expr_if_mut(&mut self, expr_if: &mut syn::ExprIf) {
        if let syn::Expr::Let(expr_let) = expr_if.cond.borrow_mut() {
            self.scope(|this| {
                this.visit_attributes_mut(&mut expr_let.attrs);
                this.visit_expr_mut(expr_let.expr.borrow_mut());
                this.visit_pat_mut(expr_let.pat.borrow_mut());
                this.visit_block_mut(&mut expr_if.then_branch);
            });
            if let Some((_, else_branch)) = &mut expr_if.else_branch {
                self.visit_expr_mut(else_branch.borrow_mut());
            }
        } else {
            visit_expr_if_mut(self, expr_if)
        }
    }
    fn visit_expr_let_mut(&mut self, _expr_let: &mut syn::ExprLet) {
        panic!("Unsupported `let` guard!")
    }
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        self.scope(|this| visit_block_mut(this, block))
    }
    fn visit_expr_closure_mut(&mut self, i: &mut syn::ExprClosure) {
        self.scope(|this| {
            this.visit_attributes_mut(&mut i.attrs);
            for input in &mut i.inputs {
                this.visit_pat_mut(input);
            }
            this.visit_expr_mut(i.body.borrow_mut());
        });
    }
    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        self.scope(|this| {
            this.visit_attributes_mut(&mut arm.attrs);
            this.visit_pat_mut(&mut arm.pat);
            if let Some((_, expr)) = &mut arm.guard {
                this.visit_expr_mut(expr.borrow_mut());
            }
            this.visit_expr_mut(arm.body.borrow_mut());
        })
    }
}

#[derive(Debug, Default)]
pub struct PartialCompute {
    bindings: Vec<(syn::Pat, syn::Expr)>,
    compute_queue: Vec<(String, proc_macro2::TokenStream)>,
}

impl PartialCompute {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_nodes(self) -> Vec<(String, proc_macro2::TokenStream)> {
        self.compute_queue
    }

    fn partial_compute(&mut self, expr: &syn::Expr) -> syn::Expr {
        let bindings = self.bindings.clone();

        fn free_vars_of_expr(expr: &syn::Expr) -> HashSet<syn::Ident> {
            let mut visitor = CollectedIntroducedVariables::new(());
            visitor.visit_expr_mut(&mut expr.clone());
            visitor.free_vars
        }
        fn free_vars_of_pat(pat: &syn::Pat) -> HashSet<syn::Ident> {
            let mut visitor = CollectedIntroducedVariables::new(());
            visitor.visit_pat_mut(&mut pat.clone());
            visitor.bound_vars
        }

        let bindings: Vec<_> = bindings
            .into_iter()
            .rev()
            .scan(free_vars_of_expr(expr), |used_vars, (lhs, rhs)| {
                Some({
                    let lhs_vars = free_vars_of_pat(&lhs);
                    if lhs_vars.intersection(&used_vars).next().is_some() {
                        let rhs_used_vars = free_vars_of_expr(&rhs).into_iter();
                        used_vars.extend(rhs_used_vars);
                        Some((lhs, rhs))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .rev()
            .collect();

        let bindings: proc_macro2::TokenStream = bindings
            .iter()
            .map(|(pat, expr)| quote! { let #pat = #expr; })
            .collect();

        let placeholder_name = format!("__testify_placeholder__{:#?}", self.compute_queue.len());
        let placeholder_ident = syn::Ident::new(&placeholder_name, proc_macro2::Span::call_site());

        let node = quote! {
            #bindings
            #expr
        };

        self.compute_queue.push((placeholder_name, node));

        parse_quote! {#placeholder_ident}
    }
}

impl PartialCompute {
    fn with_snapshot<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let bindings_snapshot = self.bindings.clone();
        let result = f(self);
        self.bindings = bindings_snapshot;
        result
    }
}
impl VisitMut for PartialCompute {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if let Ok(call) = NAryCall::<1>::try_from(&*expr) {
            let [arg] = &call.args;
            if call.func.is_ident("eval") {
                *expr = self.partial_compute(arg);
                return;
            }
        }
        visit_expr_mut(self, expr)
    }
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_local_mut(self, local);
        if let Some(local_init) = &mut local.init {
            self.bindings
                .push((local.pat.clone(), *local_init.expr.clone()));
        };
    }
    fn visit_expr_match_mut(&mut self, expr_match: &mut syn::ExprMatch) {
        let rhs: syn::Expr = (*expr_match.expr).clone();
        self.with_snapshot(|this| {
            for arm in &mut expr_match.arms {
                let lhs: syn::Pat = arm.pat.clone();
                this.bindings.push((lhs.clone(), rhs.clone()));
                this.visit_arm_mut(arm);
            }
        })
    }
    fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {
        self.with_snapshot(|this| this.visit_block_mut(&mut i.then_branch));
        if let Some((_, else_branch)) = &mut i.else_branch {
            self.with_snapshot(|this| this.visit_expr_mut(else_branch.borrow_mut()));
        }
    }
}

// #[derive(Debug, Clone, Default)]
// pub struct VariableCollector {
//     /// Keeps track of the local context
//     pub context: HashSet<syn::Ident>,
//     /// Free variables found, with their current context
//     pub free_variables: HashMap<syn::Ident, HashSet<syn::Ident>>,
// }

// impl<'a> Visit<'a> for VariableCollector {
//     fn visit_pat(&mut self, _pat: &syn::Pat) {
//         panic!("Reached `visit_pat`")
//     }
//     // fn visit_fn_arg(&mut self, fn_arg: syn::FnArg) {

//     // }
//     // fn visit_path(&mut self, path: &'a syn::Path) {
//     //     if let Some(ident) = path.expect_ident() {
//     //         self.idents.insert(ident);
//     //     };
//     //     // We need to recurse under paths, e.g. `f::<x>` refers to
//     //     // both `f` and `x`
//     //     visit_path(self, path);
//     // }
// }

// // impl IdentCollector {
// //     pub fn idents(mut self) -> HashSet<syn::Ident> {
// //         self.idents
// //     }
// // }
