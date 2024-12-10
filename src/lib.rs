mod subst;

mod complex_input_value;
pub mod driver;
mod krate;
pub mod llm;
pub mod pool;
pub mod prelude;
mod utils;

use crate::prelude::*;

pub type InputName = String;

use quote::ToTokens;

#[derive(fmt_derive::Debug, Clone, Serialize, Deserialize)]
pub enum InputKind {
    Value {
        #[debug("{}", typ.into_token_stream())]
        #[serde(with = "serde_via::SerdeVia")]
        typ: syn::Type,
        aliases: Vec<InputName>,
    },
    Type {
        #[debug("{}", bounds.into_token_stream())]
        #[serde(with = "serde_via::SerdeVia")]
        bounds: syn::WhereClause,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub name: InputName,
    pub kind: InputKind,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: proc_macro2::LineColumn,
    pub bytes: usize,
    pub file: Option<PathBuf>,
}
impl Span {
    pub fn dummy() -> Self {
        Self {
            start: proc_macro2::LineColumn { line: 0, column: 0 },
            bytes: 0,
            file: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DependencySpec(toml::Value);

impl Default for DependencySpec {
    fn default() -> Self {
        Self(toml::Value::Table(toml::Table::default()))
    }
}

#[derive(fmt_derive::Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    #[serde(default)]
    pub inputs: Vec<Input>,
    pub description: String,
    #[debug("{}", precondition.into_token_stream())]
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default = "default_expr")]
    pub precondition: syn::Expr,
    #[debug("{}", postcondition.into_token_stream())]
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default = "default_expr")]
    pub postcondition: syn::Expr,
    #[serde(default = "Span::dummy")]
    #[serde(with = "serde_via::SerdeVia")]
    pub span: Span,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default)]
    pub use_statements: Vec<syn::ItemUse>,
    #[serde(with = "serde_via::SerdeVia")]
    pub function_tested: Option<syn::Path>,
}

impl Contract {
    fn dependencies_compatible_with(&self, other: &Self) -> bool {
        let common_dependencies: HashSet<&String> =
            HashSet::from_iter(self.dependencies.keys().chain(other.dependencies.keys()));
        common_dependencies
            .into_iter()
            .all(|k| self.dependencies.get(k) == other.dependencies.get(k))
    }
    pub fn as_assertion(&self) -> proc_macro2::TokenStream {
        let postcondition = &self.postcondition;
        quote! { assert!(#postcondition); }
    }
    pub fn is_default(&self) -> bool {
        let default_expr = &default_expr();
        self.inputs.is_empty()
            && &self.precondition == default_expr
            && &self.postcondition == default_expr
    }
    pub fn normalize_paths(&mut self) {
        let workdir = std::env::current_dir().unwrap();
        for (_, DependencySpec(toml)) in self.dependencies.iter_mut() {
            let Some(path) = toml.get_mut("path") else {
                continue;
            };
            let Some(str_path) = path.as_str() else {
                continue;
            };
            *path = toml::Value::String(
                workdir
                    .join(PathBuf::from(str_path))
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            );
        }
    }
}

impl<V: ETSubst> Subst<Input> for V {
    fn subst(input: &mut Input, binding: &str, replacement: Self) {
        match &mut input.kind {
            InputKind::Value { typ, .. } => typ.subst(binding, replacement),
            InputKind::Type { bounds } => bounds.subst(binding, replacement),
        }
    }
}

#[derive(Clone)]
pub enum InputInstance {
    ComplexValue(ComplexInputValue),
    SimpleValue(syn::Expr),
    SimpleType(syn::Type),
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Precondition {
    pub inputs: Vec<(syn::Ident, syn::Type)>,
    pub predicate: syn::Expr,
}

impl Contract {
    fn function_tested(&self) -> Option<Vec<String>> {
        Some(
            self.function_tested
                .as_ref()?
                .segments
                .iter()
                .inspect(|segment| assert_eq!(segment.arguments, syn::PathArguments::None))
                .map(|segment| format!("{}", segment.ident))
                .collect(),
        )
    }

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
        self.precondition.subst(name, value.clone());
        self.postcondition.subst(name, value);
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

    pub fn expect_concrete_inputs(&self) -> Option<Vec<(syn::Ident, syn::Type)>> {
        self.inputs
            .iter()
            .map(|input| match &input.kind {
                InputKind::Value { typ, .. } => Some((
                    syn::Ident::new(&input.name, proc_macro2::Span::call_site()),
                    typ.clone(),
                )),
                _ => None,
            })
            .collect::<Option<_>>()
    }

    pub fn precondition(&self) -> Option<Precondition> {
        let inputs = self.expect_concrete_inputs()?;
        let predicate = self.precondition.clone();
        Some(Precondition { inputs, predicate })
    }

    /// Extracts the `eval` nodes
    pub fn extract_eval_nodes(&mut self) -> Vec<(String, proc_macro2::TokenStream)> {
        let mut visitor = PartialCompute::new();
        visitor.visit_expr_mut(&mut self.postcondition);
        visitor.visit_expr_mut(&mut self.precondition);
        visitor.get_nodes()
    }

    // pub fn finalize(&mut self) {
    //     let mut visitor = runner::lazy::Visitor::new().unwrap();
    //     visitor.visit_expr_mut(&mut self.postcondition);
    //     visitor.visit_expr_mut(&mut self.precondition);
    // }

    pub fn subst_names_with_exprs(&mut self, substs: HashMap<String, syn::Expr>) {
        struct Visitor(HashMap<String, syn::Expr>);
        impl VisitMut for Visitor {
            fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
                syn::visit_mut::visit_expr_mut(self, i);
                use crate::subst::syn_utils::ExpectIdent;
                if let Some(expr) = i
                    .expect_ident()
                    .and_then(|name| self.0.get(&format!("{name}")))
                    .cloned()
                {
                    *i = expr;
                }
            }
        }
        let mut visitor = Visitor(substs);
        visitor.visit_expr_mut(&mut self.postcondition);
        visitor.visit_expr_mut(&mut self.precondition);
    }
}

#[test]
fn concretization() {
    let x = Input {
        name: "x".to_string(),
        kind: InputKind::Value {
            typ: syn::parse_quote! {u8},
            aliases: vec!["x1".into()],
        },
    };
    let y = Input {
        name: "y".to_string(),
        kind: InputKind::Value {
            typ: syn::parse_quote! {u8},
            aliases: vec![],
        },
    };
    let mut contract = Contract {
        inputs: vec![x, y],
        description: "".to_string(),
        precondition: syn::parse_quote! {{
            x > 10
            // x + y
            // if let x = x {
            //     x + x
            // };
            // let a = x;
            // let e = a;
            // let b = a;
            // let d = b;
            // let c = b;
            // x + {
            //     let x = x;
            //     x
            // } + x2 + eval(x1 + c) > 3 * y()
        }},
        postcondition: syn::parse_quote! {x + 1 == eval(x + 1)},
        span: Span::dummy(),
        dependencies: HashMap::new(),
        function_tested: None,
        use_statements: vec![],
    };

    for pool in pool::ContractPool::new_pools(vec![contract]) {
        let pool = pool.instantiate_types();
        let mut pool = pool.instantiate_values();
        println!("contracts = {:#?}", pool.contracts());
        pool.compute_eval_nodes();
        println!("evaluated contracts = {:#?}", pool.contracts());
    }
    // println!("A: {:#?}", contract);
    // let precondition = contract.precondition().unwrap();
    // precondition.register();
    // // precondition.test(&["123".into(), "123".into()]);
    // let mut contracts = contract.instantiate_inputs_rand();
    // contracts.iter_mut().for_each(Contract::partial_compute);
    // contracts.iter_mut().for_each(Contract::finalize);
    // println!("{:#?}", contracts);
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
