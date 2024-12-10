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

/// Represents the kind of input used within a `Contract`. Inputs can be either a value
/// (with a type and optional aliases) or a type (with `WhereClause` bounds).
#[derive(fmt_derive::Debug, Clone, Serialize, Deserialize)]
pub enum InputKind {
    /// A value input has a `syn::Type` and zero or more aliases (alternative names) by which
    /// this input can be referenced. During instantiation, these names will be substituted
    /// with concrete values.
    Value {
        #[debug("{}", typ.into_token_stream())]
        #[serde(with = "serde_via::SerdeVia")]
        typ: syn::Type,
        aliases: Vec<InputName>,
    },
    /// A type input is represented by a `syn::WhereClause` and can be replaced by a concrete
    /// type bound at instantiation.
    Type {
        #[debug("{}", bounds.into_token_stream())]
        #[serde(with = "serde_via::SerdeVia")]
        bounds: syn::WhereClause,
    },
}

/// A named input that forms part of a `Contract`. Each input has a name and a specific `InputKind`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    /// The name of this input, used for substitution and identification.
    pub name: InputName,
    /// The kind of input (value with type and aliases, or type with bounds).
    pub kind: InputKind,
}

/// Represents a span of code or file content. Contains line-column information, byte offset,
/// and optional file location. Useful for error reporting and diagnostics.
#[derive(Debug, Clone)]
pub struct Span {
    /// The starting line and column of this span.
    pub start: proc_macro2::LineColumn,
    /// The number of bytes this span covers.
    pub bytes: usize,
    /// The file path associated with this span, if available.
    pub file: Option<PathBuf>,
}

impl Span {
    /// Creates a "dummy" span, with no meaningful location information.
    pub fn dummy() -> Self {
        Self {
            start: proc_macro2::LineColumn { line: 0, column: 0 },
            bytes: 0,
            file: None,
        }
    }
}

/// A specification of dependencies for a `Contract`, stored as a `toml::Value`. The TOML value is expected to be similar to what Cargo expects for dependencies.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DependencySpec(toml::Value);

impl Default for DependencySpec {
    fn default() -> Self {
        Self(toml::Value::Table(toml::Table::default()))
    }
}

/// A `Contract` defines a set of inputs, a description, a precondition, and a postcondition.
/// It can also contain additional data such as dependencies, use-statements, and an optional
/// tested function. Contracts can be instantiated with concrete inputs and then evaluated.
#[derive(fmt_derive::Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// The list of inputs (value or type) that this contract expects.
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// A human-readable description of what the contract represents or enforces.
    pub description: String,
    /// A precondition expression, which must hold before the code under test is run.
    #[debug("{}", precondition.into_token_stream())]
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default = "default_expr")]
    pub precondition: syn::Expr,
    /// A postcondition expression, which must hold after the code under test has run.
    #[debug("{}", postcondition.into_token_stream())]
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default = "default_expr")]
    pub postcondition: syn::Expr,
    /// The span of the contract definition in source code, used for diagnostics.
    #[serde(default = "Span::dummy")]
    #[serde(with = "serde_via::SerdeVia")]
    pub span: Span,
    /// Dependencies required by this contract, keyed by name.
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    /// Additional `use` statements that should be in scope when evaluating the contract.
    #[serde(with = "serde_via::SerdeVia")]
    #[serde(default)]
    pub use_statements: Vec<syn::ItemUse>,
    /// The function under test, if any, represented by a `syn::Path`.
    #[serde(with = "serde_via::SerdeVia")]
    pub function_tested: Option<syn::Path>,
}

impl Contract {
    /// Checks if the dependencies of this contract are compatible with the dependencies
    /// of another contract. Two contracts are considered compatible if all common dependencies
    /// match exactly.
    fn dependencies_compatible_with(&self, other: &Self) -> bool {
        let common_dependencies: HashSet<&String> =
            HashSet::from_iter(self.dependencies.keys().chain(other.dependencies.keys()));
        common_dependencies
            .into_iter()
            .all(|k| self.dependencies.get(k) == other.dependencies.get(k))
    }

    /// Generates a token stream that asserts the postcondition. Useful for code emission.
    pub fn as_assertion(&self) -> proc_macro2::TokenStream {
        let postcondition = &self.postcondition;
        quote! { assert!(#postcondition); }
    }

    /// Checks if this `Contract` is essentially empty or default. A default contract has no inputs
    /// and default pre- and post-conditions.
    pub fn is_default(&self) -> bool {
        let default_expr = &default_expr();
        self.inputs.is_empty()
            && &self.precondition == default_expr
            && &self.postcondition == default_expr
    }

    /// Normalizes all file paths in the dependencies, converting relative paths to absolute paths
    /// based on the current working directory.
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

/// Represents a concrete instantiation of an input. Inputs can be instantiated as complex values
/// (with multiple bindings), simple values (an `syn::Expr`), or simple types (`syn::Type`).
#[derive(Clone)]
pub enum InputInstance {
    /// A complex value input, which may introduce multiple `let` bindings before using the final value.
    ComplexValue(ComplexInputValue),
    /// A simple value expression that replaces the input directly.
    SimpleValue(syn::Expr),
    /// A simple type substitution that replaces a type input directly.
    SimpleType(syn::Type),
}

/// Represents a precondition extracted from a contract. It contains a list of expected inputs
/// along with their types, and a predicate expression that must hold true.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Precondition {
    /// The inputs (by ident and type) that the precondition expects.
    pub inputs: Vec<(syn::Ident, syn::Type)>,
    /// The predicate expression that must be true given the inputs.
    pub predicate: syn::Expr,
}

impl Contract {
    /// Retrieves the tested function path segments as a vector of strings, if a function is tested.
    /// Each segment is asserted to have no path arguments.
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

    /// Removes and returns the specified input from the contract, along with an iterator over the
    /// remaining inputs after that input’s position. Useful for operations that need to remove one
    /// input and then apply transformations to subsequent inputs.
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

    /// Returns a set of concrete identifiers found in both the precondition and postcondition
    /// expressions, excluding those declared as inputs. This set can help identify free variables
    /// that must be handled or instantiated.
    pub fn concrete_idents(&self) -> HashSet<syn::Ident> {
        let mut visitor = IdentCollector::default();
        visitor.visit_expr(&self.precondition);
        visitor.visit_expr(&self.postcondition);
        visitor.idents()
    }

    /// Substitutes all occurrences of the given input name in both the precondition and postcondition
    /// with the provided `value`.
    pub fn subst_in_body(&mut self, name: &str, value: impl Subst<syn::Expr>) {
        self.precondition.subst(name, value.clone());
        self.postcondition.subst(name, value);
    }

    /// Substitutes all occurrences of a set of input aliases in both the precondition and postcondition
    /// with the provided `value`.
    pub fn multi_subst_in_body(&mut self, aliases: Vec<String>, value: impl Subst<syn::Expr>) {
        for alias in &aliases {
            self.subst_in_body(alias, value.clone())
        }
    }

    /// Instantiates an input by name with a given `InputInstance`. This involves removing the input
    /// from the contract, then substituting its occurrences in the body (pre- and post-condition).
    /// For complex values, additional `let` bindings are prepended.
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
                // For complex values, we prepend bindings rather than simple substitution.
                drop(inputs);

                let span = proc_macro2::Span::call_site();

                for name in &names {
                    let ident = syn::Ident::new(name, span);

                    // Prepend necessary bindings to introduce the complex value.
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

    /// Ensures that all inputs are concrete values, returning their identifiers and types if so.
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

    /// Extracts a `Precondition` from the contract, if all inputs are concrete.
    pub fn precondition(&self) -> Option<Precondition> {
        let inputs = self.expect_concrete_inputs()?;
        let predicate = self.precondition.clone();
        Some(Precondition { inputs, predicate })
    }

    /// Extracts `eval` nodes from the precondition and postcondition by partially computing certain
    /// parts of the expressions. Returns a vector of `(String, TokenStream)` pairs representing the
    /// extracted nodes.
    pub fn extract_eval_nodes(&mut self) -> Vec<(String, proc_macro2::TokenStream)> {
        let mut visitor = PartialCompute::new();
        visitor.visit_expr_mut(&mut self.postcondition);
        visitor.visit_expr_mut(&mut self.precondition);
        visitor.get_nodes()
    }

    /// Substitutes identifiers within the pre- and postcondition expressions with the given map of
    /// replacement expressions.
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
    let contract = Contract {
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
