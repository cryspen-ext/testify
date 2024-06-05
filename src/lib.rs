mod subst;

mod complex_input_value;

mod prelude;

use crate::prelude::*;

use crate::visitors::SubstReferences;

pub type InputName = String;

use quote::ToTokens;

#[derive(fmt_derive::Debug, Clone)]
pub enum InputKind {
    Value {
        #[debug("{}", typ.into_token_stream())]
        typ: syn::Type,
        aliases: Vec<InputName>,
    },
    Type {
        #[debug("{}", bounds.into_token_stream())]
        bounds: syn::WhereClause,
    },
}

#[derive(Debug, Clone)]
pub struct Input {
    pub name: InputName,
    pub kind: InputKind,
}

#[derive(fmt_derive::Debug, Clone)]
pub struct Contract {
    pub inputs: Vec<Input>,
    pub description: String,
    #[debug("{}", precondition.into_token_stream())]
    pub precondition: syn::Expr,
    #[debug("{}", postcondition.into_token_stream())]
    pub postcondition: syn::Expr,
}

impl<V: ETSubst> Subst<Input> for V {
    fn subst(input: &mut Input, binding: &str, replacement: Self) {
        match &mut input.kind {
            InputKind::Value { typ, .. } => typ.subst(binding, replacement),
            InputKind::Type { bounds } => bounds.subst(binding, replacement),
        }
    }
}

pub enum InputInstance {
    ComplexValue(ComplexInputValue),
    SimpleValue(syn::Expr),
    SimpleType(syn::Type),
}

trait PrependLocal {
    fn prepend_local(&mut self, local: syn::Local);
    fn prepend_binding(&mut self, span: proc_macro2::Span, lhs: syn::Pat, rhs: syn::Expr) {
        self.prepend_local(syn::Local {
            attrs: vec![],
            let_token: syn::Token![let](span),
            pat: lhs,
            init: Some(syn::LocalInit {
                eq_token: syn::Token![=](span),
                expr: Box::new(rhs),
                diverge: None,
            }),
            semi_token: syn::Token![;](span),
        })
    }
}

impl PrependLocal for syn::Block {
    fn prepend_local(&mut self, local: syn::Local) {
        self.stmts.insert(0, syn::Stmt::Local(local))
    }
}
impl PrependLocal for syn::Expr {
    fn prepend_local(&mut self, local: syn::Local) {
        match self {
            syn::Expr::Block(block) => block.block.prepend_local(local),
            _ => {
                *self = syn::parse_quote! {{
                    #local
                    #self
                }}
            }
        }
    }
}
impl PrependLocal for Contract {
    fn prepend_local(&mut self, local: syn::Local) {
        self.precondition.prepend_local(local.clone());
        self.postcondition.prepend_local(local);
    }
}

impl Contract {
    fn retain_input(&mut self, input: &str) -> (InputKind, impl Iterator<Item = &mut Input>) {
        let (idx, _) = self
            .inputs
            .iter()
            .enumerate()
            .find(|(_, i)| i.name == input)
            .expect("No input named `{binding}`");
        (
            self.inputs.remove(idx).kind,
            self.inputs.iter_mut().skip(idx),
        )
    }

    /// Returns a set of concrete idents (i.e. idents found everywhere but in the inputs: those will be instantiated at some point)
    pub fn concrete_idents(&self) -> HashSet<syn::Ident> {
        let mut visitor = IdentCollector::default();
        visitor.visit_expr(&self.precondition);
        visitor.visit_expr(&self.postcondition);
        visitor.idents()
    }
    pub fn subst_in_body(&mut self, name: &str, value: impl Subst<syn::Expr>) {
        self.precondition.subst(name, value);
    }
    pub fn multi_subst_in_body(&mut self, aliases: Vec<String>, value: impl Subst<syn::Expr>) {
        for alias in &aliases {
            self.subst_in_body(alias, value.clone())
        }
    }
    pub fn instantiate_input(&mut self, input: &str, value: InputInstance) {
        let (input_kind, inputs) = self.retain_input(input);
        let names = {
            let mut names = vec![input.to_string()];
            if let InputKind::Value { aliases, .. } = input_kind {
                names.extend_from_slice(&aliases)
            }
            names
        };
        match value {
            InputInstance::ComplexValue(complex_input_value) => {
                // complex values cannot be used in const generics
                drop(inputs);

                let span = proc_macro2::Span::call_site();

                for name in &names {
                    let ident = syn::Ident::new(name, span);
                    // let expr: syn::Expr = syn::parse_quote! {#ident};
                    // self.subst_in_body(name, expr);

                    for (pat, expr) in complex_input_value.bindings.clone().into_iter().rev() {
                        self.prepend_binding(span, pat, expr)
                    }
                    self.prepend_binding(
                        span,
                        parse_quote! {mut #ident},
                        complex_input_value.rhs.clone(),
                    )
                }
            }
            InputInstance::SimpleValue(expr) => {
                for input in inputs {
                    for name in &names {
                        input.subst(name, expr.clone());
                    }
                }
                self.multi_subst_in_body(names, expr.clone());
            }
            InputInstance::SimpleType(typ) => {
                for input in inputs {
                    for name in &names {
                        input.subst(name, typ.clone());
                    }
                }
                self.multi_subst_in_body(names, typ.clone());
            }
        };
    }
}

#[test]
fn hello() {
    let x = Input {
        name: "x".to_string(),
        kind: InputKind::Value {
            typ: syn::parse_quote! {u32},
            aliases: vec!["x1".into()],
        },
    };
    let y = Input {
        name: "y".to_string(),
        kind: InputKind::Value {
            typ: syn::parse_quote! {u32},
            aliases: vec![],
        },
    };
    let mut contract = Contract {
        inputs: vec![x, y],
        description: "".to_string(),
        precondition: syn::parse_quote! {{
            if let x = x {
                x + x
            };
            let a = x;
            let e = a;
            let b = a;
            let d = b;
            let c = b;
            x + {
                let x = x;
                x
            } + x2 + eval(x1 + c) > 3 * y()
        }},
        postcondition: syn::parse_quote! {true},
    };
    println!("A: {:#?}", contract);
    contract.instantiate_input("x", InputInstance::SimpleValue(syn::parse_quote! {42}));
    // contract.instantiate_input(
    //     "x",
    //     InputInstance::ComplexValue(ComplexInputValue {
    //         bindings: vec![],
    //         mutable: true,
    //         rhs: syn::parse_quote! {42},
    //     }),
    // );
    println!("B: {:#?}", contract);
    let mut visitor = PartialCompute::new();
    visitor.visit_expr_mut(&mut contract.precondition);
    println!("C: {:#?}", contract);
}
