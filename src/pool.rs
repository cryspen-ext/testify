use crate::krate::{
    run_or_locate_error,
    server::{declare, Server},
    Krate,
};
use crate::prelude::*;
use crate::Contract;
use hax_frontend_exporter::{Ty, TyKind};

/// Declares the types that represent every possible state a pool of
/// contract can be in.
mod state {
    use super::*;
    use crate::krate::hax::*;

    /// A pool that contains contracts with universally quantified types
    #[derive(Copy, Clone, Debug)]
    pub struct GenericContracts;

    /// A pool that contains contracts with universally quantified values
    pub struct ParametricContracts {
        precondition_server: Server,
        pub types: Vec<Vec<Ty>>,
    }
    declare! {
        Api,
        pub mod api {
            #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
            pub struct Input {
                pub id: usize,
                pub contents: ::serde_json::Value,
            }
            pub type Output = Option<bool>;
        }
    }

    impl ParametricContracts {
        /// Test the precondition of the nth contract given
        /// JSON-encoded inputs. Returns `Some(r)` with `r` the result
        /// of the precondition, or `None` if compiling or executing
        /// the precondition panicked.
        pub fn test_precondition(
            &mut self,
            nth: usize,
            inputs: Vec<serde_json::Value>,
        ) -> api::Output {
            let contents = serde_json::Value::Array(inputs);
            self.precondition_server
                .request_json(&api::Input { id: nth, contents })
        }

        /// Create a `ParametricContracts` structs: this uses hax to
        /// resolve the input types of the contracts, and sets up a
        /// precondition server.
        pub fn new(contracts: &[Contract], deps: &HashMap<String, DependencySpec>) -> Self {
            assert!(contracts.iter().all(Self::check));

            let types = {
                let queries: Vec<Vec<_>> = contracts
                    .iter()
                    .map(|contract| {
                        let pre = &contract.precondition().unwrap();
                        let types = pre.inputs.iter().map(|(_, typ)| typ);
                        types
                            .cloned()
                            .map(|typ| HaxQuery::Type {
                                generics: parse_quote! {<>},
                                typ,
                                use_statements: contract.use_statements.clone(),
                            })
                            .collect()
                    })
                    .collect();

                let raw_types = execute_hax_queries(
                    &queries.into_iter().flatten().collect::<Vec<_>>()[..],
                    deps,
                )
                .unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    panic!()
                });
                let mut i = 0;
                let mut types = vec![];
                for contract in contracts {
                    let mut contract_types = vec![];
                    let pre = &contract.precondition().unwrap();
                    for _ in pre.inputs.iter() {
                        contract_types.push(match &raw_types[i] {
                            HaxQueryRes::Type(ty) => ty.clone(),
                        });
                        i += 1;
                    }
                    types.push(contract_types);
                }
                types
            };

            let precondition_server = {
                let arms = contracts.iter().enumerate().map(|(i, contract)| {
                    let pre = &contract.precondition().unwrap();
                    let predicate = &pre.predicate;
                    let types = &pre.inputs.iter().map(|(_, typ)| typ).collect::<Vec<_>>();
                    let names = &pre.inputs.iter().map(|(name, _)| name).collect::<Vec<_>>();
                    let use_statements = &contract.use_statements;
                    quote! {
                        #i => {
                            let arena = ::marshalling::Arena::new();
                            use ::marshalling::FromValueRepr as _;
                            type INPUTS = (#(#types,)*);
                            let serde_json::Value::Array(vec) = request else {panic!("Expected a JSON array")};
                            let [#(#names,)*] = &vec[..] else {panic!("Bad number of inputs")};
                            let (#(#names,)*): INPUTS = (#(<#types>::from_value_repr(&#names, &arena),)*);
                            let response: api::Output = ::std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                #(use #use_statements;)*
                                #predicate
                            })).ok();
                            response
                        }
                    }
                });

                Server::from_json_fn(
                    quote! {
                        #Api
                        let api::Input {id, contents} = request;
                        let request = contents;
                        #[allow(warning, unused)]
                        {
                            match id {
                                #(#arms)*
                                _ => panic!(),
                            }
                        }
                    },
                    deps,
                )
            };

            Self {
                precondition_server,
                types,
            }
        }
    }

    /// A pool that contains fully instantiated contracts
    #[derive(Copy, Clone, Debug)]
    pub struct InstantiatedContracts;

    impl IsState for GenericContracts {
        fn check(_contract: &crate::Contract) -> bool {
            true
        }
    }
    impl IsState for ParametricContracts {
        fn check(contract: &crate::Contract) -> bool {
            !contract
                .inputs
                .iter()
                .any(|input| matches!(input.kind, crate::InputKind::Type { .. }))
        }
    }
    impl IsState for InstantiatedContracts {
        fn check(contract: &crate::Contract) -> bool {
            contract.inputs.is_empty()
        }
    }

    pub trait IsState {
        fn check(contract: &crate::Contract) -> bool;
        fn assert(contract: &crate::Contract) {
            assert!(Self::check(contract))
        }
    }
}
pub use state::*;

#[derive(Debug)]
pub struct ContractPool<State: IsState> {
    contracts: Vec<crate::Contract>,
    state: State,
    // TODO: add the Cargo dependencies needed for the contracts. This
    // should come in two sets: first, the set of crates necessary for
    // running the tests, second, the set of crates necessary for the
    // abstractions used by the contracts.
}

declare! {
    TypeMarshalling,
    mod type_marshalling {
        use ::serde_json::Value;
        use ::serde::de::DeserializeOwned;
        trait HasMarhsalling {
            fn decode(value: Value) -> Self;
        }
        macro_rules! impl_marshalling_via_serde {
            ( $t:ty $({{$($generics:tt)*} $($where:tt)*})? $(, $($tt:tt)*)? ) => {
                impl<$($($generics)*)?> HasMarhsalling for $t $($($where)*)? {
                    fn decode(value: Value) -> Self {
                        ::serde_json::from_value(value).unwrap()
                    }
                }
                $(impl_marshalling_via_serde!($($tt)*);)?
            };
            () => {}
        }

        macro_rules! impl_tuple_marshalling_via_serde {
            ($t:ident) => {};
            (@) => {};
            (@$a:ident $(, $t:ident)*) => {
                impl_tuple_marshalling_via_serde!(@$($t),*);
                impl_tuple_marshalling_via_serde!($a $(,$t)*);
            };
            ($($t:ident),*) => {
                impl_marshalling_via_serde!(($($t),*) {{$($t: HasMarhsalling + DeserializeOwned),*}});
            }
        }

        impl_marshalling_via_serde!(u8, u16, u32, u64, u128, usize);
        impl_marshalling_via_serde!(i8, i16, i32, i64, i128, isize);
        impl_marshalling_via_serde!((), bool);
        impl_marshalling_via_serde!(Option<T> {{T: HasMarhsalling + DeserializeOwned}});
        impl_marshalling_via_serde!(Vec<T> {{T: HasMarhsalling + DeserializeOwned}});

        impl_tuple_marshalling_via_serde!(@A, B, C, D, E, F, G, H, I);
    }
}

impl ContractPool<GenericContracts> {
    /// Creates a fresh pool
    pub fn new_pools(contracts: Vec<crate::Contract>) -> Vec<Self> {
        let mut groups: Vec<(crate::Contract, Vec<crate::Contract>)> = vec![];
        for contract in contracts {
            if let Some((repr, group)) = groups
                .iter_mut()
                .find(|(candidate, _)| candidate.dependencies_compatible_with(&contract))
            {
                let dependencies: HashMap<_, _> = contract
                    .dependencies
                    .iter()
                    .map(|(name, version)| (name.clone(), version.clone()))
                    .chain(repr.dependencies.drain())
                    .collect();
                repr.dependencies = dependencies.clone();
                group.push(contract);
            } else {
                groups.push((contract.clone(), vec![contract]));
            }
        }
        groups
            .into_iter()
            .map(|(_, group)| Self::new(group))
            .collect()
    }

    /// Creates a fresh pool, assuming all contracts have compatible dependencies
    fn new(contracts: Vec<crate::Contract>) -> Self {
        Self {
            contracts,
            state: GenericContracts,
        }
    }

    pub fn instantiate_types(self) -> ContractPool<ParametricContracts> {
        let state = ParametricContracts::new(&self.contracts, &self.dependencies());
        self.retype(state)
            .expect("Generic contracts are not supported yet")
    }
}

pub fn arbitrary<T: for<'a> arbitrary::Arbitrary<'a>>() -> T {
    let mut rng = rand::thread_rng();
    let raw_data: &mut [u8] = &mut [0; 512];
    rng.fill_bytes(raw_data);
    // use arbitrary::Arbitrary as _;
    use arbitrary::Unstructured;
    use rand::RngCore;
    T::arbitrary(&mut Unstructured::new(raw_data)).unwrap()
}

use hax_frontend_exporter::{IntTy, UintTy};
// This should generate an AST
fn generate_for_type(ty: &Ty) -> (serde_json::Value, String) {
    use marshalling::*;
    fn rand<T: ToValueRepr + ToRustExpr + for<'a> arbitrary::Arbitrary<'a>>(
    ) -> (serde_json::Value, String) {
        let value = arbitrary::<T>();
        let repr = value.to_value_repr();
        let expr = value.to_rust_expr();
        (repr, expr)
    }
    match ty.kind() {
        TyKind::Uint(UintTy::U8) => rand::<u8>(),
        TyKind::Uint(UintTy::U16) => rand::<u16>(),
        TyKind::Uint(UintTy::U32) => rand::<u32>(),
        TyKind::Uint(UintTy::U64) => rand::<u64>(),
        TyKind::Uint(UintTy::U128) => rand::<u128>(),
        TyKind::Uint(UintTy::Usize) => rand::<usize>(),
        TyKind::Int(IntTy::I8) => rand::<i8>(),
        TyKind::Int(IntTy::I16) => rand::<i16>(),
        TyKind::Int(IntTy::I32) => rand::<i32>(),
        TyKind::Int(IntTy::I64) => rand::<i64>(),
        TyKind::Int(IntTy::I128) => rand::<i128>(),
        TyKind::Int(IntTy::Isize) => rand::<isize>(),
        // Ty::Slice(ty) => {
        //     let n = arbitrary::<usize>() % 6;
        //     let values: Vec<_> =
        //         [(); 6].map(|()| generate_for_type(ty)).into_iter().collect();
        //     (serde_json::Value::Array(reprs), format!("[]"))
        // }
        // Ty::Tuple(types) => serde_json::Value::Array(types.iter().map(generate_for_type).collect()),
        _ => todo!("Unsupported type {ty:?}"),
    }
}

impl ContractPool<ParametricContracts> {
    pub fn instantiate_values(mut self) -> ContractPool<InstantiatedContracts> {
        let mut instantiated_contracts = vec![];
        for (i, contract) in self.contracts.iter().enumerate() {
            let mut instances = vec![];
            for _ in 1..(contract.tests * 20) {
                if instances.len() >= contract.tests {
                    break;
                }
                let types = &self.state.types[i];
                let values = {
                    types
                        .iter()
                        .map(generate_for_type)
                        .collect::<Vec<(serde_json::Value, String)>>()
                };
                let Some(result) = self.state.test_precondition(
                    i,
                    values.clone().into_iter().map(|(repr, _)| repr).collect(),
                ) else {
                    panic!("Precondition panicked!")
                };
                if result {
                    let mut new_contract = contract.clone();
                    assert!(&values.len() == &contract.inputs.len());
                    for ((_, rust_expr), input) in values.iter().zip(contract.inputs.iter()) {
                        new_contract.instantiate_input(
                            &input.name,
                            crate::InputInstance::SimpleValue(syn::parse_str(&rust_expr).unwrap()),
                        );
                    }
                    instances.push(new_contract);
                }
            }
            instantiated_contracts.extend(instances);
        }
        ContractPool {
            contracts: instantiated_contracts,
            state: InstantiatedContracts,
        }
    }
}

fn eval_expressions(
    exprs: &[proc_macro2::TokenStream],
    dependencies: &HashMap<String, DependencySpec>,
) -> Result<Vec<Result<String, String>>, (String, String)> {
    declare! {
        Api,
        pub mod api {
            pub type Output = Vec<Result<String, String>>;
        }
    }
    let mut krate = Krate::new();
    krate.add_dependencies(dependencies);
    let n = exprs.len();
    let program = quote! {
        #Api

        use marshalling::ToRustExpr;
        fn main() {
            let functions: [fn() -> String; #n] = [#({
                (|| {#exprs}.to_rust_expr())
            }),*];
            let results: api::Output = functions.into_iter().map(|function| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    function()
                })).map_err(|err| format!("{:#?}", err))
            }).collect();
            println!("{}", serde_json::to_string(&results).unwrap())
        }
    };
    krate.use_serde();
    let program = format!("{}", program.to_token_stream());
    krate.source(&program);

    let output = krate.run().wait_with_output().unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();

    if output.status.success() {
        Ok(serde_json::from_str(&stdout).expect(stdout))
    } else {
        Err((stderr.into(), program))
    }
}

impl ContractPool<InstantiatedContracts> {
    pub fn compute_eval_nodes(&mut self) {
        let mut identifiers: Vec<Vec<_>> = vec![];
        let mut nodes: Vec<_> = vec![];
        for contract in &mut self.contracts {
            let eval_nodes = contract.extract_eval_nodes();
            let (contract_ids, contract_nodes): (Vec<_>, Vec<_>) = eval_nodes.into_iter().unzip();
            identifiers.push(contract_ids);
            let use_statements = &contract.use_statements;
            let contract_nodes: Vec<_> = contract_nodes
                .into_iter()
                .map(|node| {
                    quote! {
                        #(use #use_statements;)*
                        #node
                    }
                })
                .collect();
            nodes.extend(contract_nodes);
        }

        if nodes.is_empty() {
            return;
        }

        let dependencies = self.dependencies();
        let nodes = run_or_locate_error(&nodes, |node| eval_expressions(node, &dependencies))
            .unwrap_or_else(|(context, (stderr, program))| {
                eprintln!(
                    "{:#?}",
                    context
                        .iter()
                        .map(|x| format!("{}", x.into_token_stream()))
                        .collect::<Vec<_>>()
                );
                eprintln!("> stderr: {stderr}");
                eprintln!("> program: {program}");
                panic!()
            });

        let mut cursor = 0;
        for (contract, identifiers) in self.contracts.iter_mut().zip(identifiers.iter()) {
            let mut substs = HashMap::new();
            for identifier in identifiers {
                let node = nodes[cursor].clone().unwrap();
                substs.insert(identifier.clone(), syn::parse_str(&node).unwrap());
                cursor += 1;
            }
            contract.subst_names_with_exprs(substs);
        }
    }

    /// Computes coverage information for a crate using both `tarpaulin` and `hax`.
    ///
    /// This function combines data from two tools to generate precise
    /// metrics:
    ///
    /// - **`tarpaulin`**: Provides line-by-line coverage information
    /// (covered/uncovered) for all files within a crate. However,
    /// this raw data includes irrelevant coverage details outside the
    /// scope of the function being tested. For example, when testing
    /// a function `double`, we are only interested in the coverage of
    /// the definition of function `double` itself, not in the
    /// coverage of other function implementations (e.g. `+`), which
    /// are indirectly invoked.
    ///
    /// - **`hax`**: Supplies precise span information for the
    /// specific function being tested. Using these spans, we filter
    /// the output from `tarpaulin` to focus only on the relevant
    /// parts of the code under test.
    ///
    /// ### Workflow
    /// 1. For each function tested by the contracts:
    ///    - Extract the associated crate and resolve the path to its source.
    ///    - Duplicate the crate to safely inject additional test cases and formatting changes.
    /// 2. Query `hax` to determine the precise span of the function under test.
    /// 3. Use the span to:
    ///    - Insert a unit test for the function directly after its definition.
    ///    - Format the crate to align the control-flow branches for better per-line coverage analysis.
    /// 4. Run `tarpaulin` to generate a coverage report filtered to the span of the function under test.
    ///
    /// ### Returns
    /// A vector of `BadCoverageReport`, each representing uncovered lines for the
    /// functions tested by the contracts.
    #[tracing::instrument]
    pub fn compute_coverage(&self) -> Vec<crate::krate::tarpaulin::BadCoverageReport> {
        let by_functions_tested: HashMap<Vec<String>, Vec<&Contract>> = self
            .contracts
            .iter()
            .flat_map(|contract| Some((contract.function_tested()?, contract)))
            .into_group_map();

        by_functions_tested
            .into_iter()
            .filter_map(|(fn_path, contracts)| {
                trace!("fn_path={:?}", fn_path);
                // The contracts in `contracts` are all about the same
                // function `fn_path`.

                // The crate of the function we're testing
                let krate_name = &fn_path[0];

                let assertions: Vec<_> = contracts
                    .iter()
                    .map(|contract| contract.as_assertion())
                    .collect();

                let krate = {
                    // Find the full path to the source of the crate `krate_name`.
                    let krate_path = {
                        // `krate` is a dummy crate whose dependencies
                        // are matching dependencies declared in the
                        // contracts `contracts`
                        let krate = {
                            let mut krate = Krate::new();
                            krate.add_dependencies(&self.dependencies());
                            krate
                        };
                        // Runs `cargo metadata`, and finds the path
                        // to the `Cargo.toml` of the crate `krate_name`.
                        let manifest_path = krate
                            .manifest_path_of_crate(krate_name)
                            .expect(&format!("Could not find {krate_name}: please make sure your contract declares a dependency on that crate. The dependencies currently available are: {:#?}.", self.dependencies()));
                        // Returns the parent folder of the `Cargo.toml` manifest
                        manifest_path.parent().unwrap().to_path_buf()
                    };
                    // We duplicate the crate `krate_name` so that we can edit it freely
                    Krate::duplicate_crate(&krate_path, &self.dependencies().into_iter().collect())
                        .unwrap()
                };

                // Stringify the path
                let fn_path = fn_path.join("::");

                // Ask hax about the span of the item `fn_path`
                let span = {
                    let items = krate.hax().unwrap();
                    let item = items
                        .iter()
                        .find(|item| {
                            let mut owner_id =
                                (&item.owner_id as &hax_frontend_exporter::DefIdContents).clone();
                            owner_id.krate = krate_name.to_string();
                            owner_id.into_string() == fn_path
                        })
                        .unwrap();
                    item.span.clone()
                };

                // Reconstruct the full path to the Rust file holding the item `fn_path`
                let filepath = krate
                    .workspace_path()
                    .join(span.filename.to_path().unwrap());

                // Construct a unit test that runs the assertions
                let test_function = {
                    let test = quote! {
                        #[test]
                        fn testify_test() {
                            #(#assertions)*
                        }
                    };
                    let test = format!("{}", test.to_token_stream());
                    test.replace(krate_name, "crate")
                };

                // Insert the unit test right after the item `fn_path` in place
                {
                    use std::fs;
                    let contents = fs::read_to_string(&filepath).unwrap();
                    let (before, after) = contents.split_at_loc(span.hi.clone());
                    fs::write(&filepath, format!("{before}{test_function}{after}")).unwrap();
                }

                // Format the crate: we want the various control-flow
                // branches to be on their own line, `tarpaulin` gives
                // a per-line report
                krate.fmt().unwrap();

                // Ask tarpaulin a coverage report, but keep only the
                // reports that are within the span `span`
                let report =
                    krate
                        .tarpaulin()
                        .coverage_for_span(fn_path.to_string(), &filepath, span);
                trace!("report={:?}", report);
                report
            })
            .inspect(|report| println!("{report}"))
            .collect()
    }
}

impl<State: IsState> ContractPool<State> {
    fn retype<NextState: IsState>(self, state: NextState) -> Option<ContractPool<NextState>> {
        let Self {
            contracts,
            state: _,
        } = self;
        let pool = ContractPool { contracts, state };
        pool.check().then_some(pool)
    }
    pub fn check(&self) -> bool {
        self.contracts.iter().all(State::check)
    }

    pub fn assert(&self) {
        assert!(self.check())
    }

    pub fn contracts(&self) -> &[Contract] {
        &self.contracts[..]
    }

    /// Returns the Cargo dependencies for this pool. This function
    /// assumes contracts dependencies are compatible. If this
    /// invariant is not met, the function panics.
    pub fn dependencies(&self) -> HashMap<String, DependencySpec> {
        let mut deps = HashMap::new();
        for contract in self.contracts() {
            for (dependency, version) in &contract.dependencies {
                if let Some(previous_version) = deps.insert(dependency.to_string(), version.clone())
                {
                    assert_eq!(&previous_version, version);
                }
            }
        }
        deps
    }
    pub fn dependencies_string(&self) -> String {
        dependencies_to_string(&self.dependencies())
    }
}
