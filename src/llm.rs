use crate::krate::Krate;
use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptContext {
    pub tested_item: String,
    pub tested_item_contents: String,
    pub related_items_contents: Vec<String>,
    pub related_contracts: Vec<Contract>,
}

impl Contract {
    /// Creates an empty, dummy contract.
    fn dummy() -> Self {
        Contract {
            inputs: vec![],
            description: String::new(),
            precondition: parse_quote! {true},
            postcondition: parse_quote! {true},
            span: Span::dummy(),
            dependencies: HashMap::new(),
            use_statements: vec![],
            function_tested: None,
        }
    }
}

impl PromptContext {
    pub fn new(
        dependencies: HashMap<String, DependencySpec>,
        item_to_test: syn::Path,
        contracts: Vec<Contract>,
    ) -> PromptContext {
        let mut contract = Contract::dummy();
        contract.dependencies = dependencies;
        contract.function_tested = Some(item_to_test);
        let function_tested = contract.function_tested().unwrap();
        let krate_name = &function_tested[0];
        let krate = {
            // Find the full path to the source of the crate `krate_name`.
            let krate_path = {
                // `krate` is a dummy crate whose dependencies
                // are matching dependencies declared in the
                // contracts `contracts`
                let mut krate = {
                    let mut krate = Krate::new();
                    krate.add_dependencies(&contract.dependencies);
                    krate
                };
                // Runs `cargo metadata`, and finds the path
                // to the `Cargo.toml` of the crate `krate_name`.
                let manifest_path = krate.manifest_path_of_crate(krate_name).unwrap();
                // Returns the parent folder of the `Cargo.toml` manifest
                manifest_path.parent().unwrap().to_path_buf()
            };
            // We duplicate the crate `krate_name` so that we can edit it freely
            Krate::duplicate_crate(
                &krate_path,
                &contract.dependencies.clone().into_iter().collect(),
            )
            .unwrap()
        };

        let items = krate.hax().unwrap();
        let def_id_to_string = |did: &hax_frontend_exporter::DefId| {
            let mut did = (&did as &hax_frontend_exporter::DefIdContents).clone();
            if did.krate == krate.name() {
                did.krate = krate_name.to_string();
            }
            did.into_string()
        };
        let find_item = |fn_path: String| {
            items
                .iter()
                .find(|item| def_id_to_string(&item.owner_id) == fn_path)
        };
        let function_tested = function_tested.join("::");
        trace!("function_tested={function_tested}");
        trace!(
            "items={:#?}",
            items
                .iter()
                .map(|item| def_id_to_string(&item.owner_id))
                .collect::<Vec<_>>()
        );
        let item = find_item(function_tested.clone()).unwrap();
        let workdir = krate.workspace_path();
        let related_items_ids = item.def_ids();
        let related_items_contents: Vec<String> = related_items_ids
            .iter()
            .map(def_id_to_string)
            .flat_map(find_item)
            .filter(|i| def_id_to_string(&i.owner_id) != function_tested)
            .map(|i| i.span.clone())
            .unique()
            .flat_map(|span| span.source(&workdir))
            .collect();
        trace!("item.span={:#?}", item.span);
        let tested_item_contents = item.span.source(&workdir).unwrap();
        let related_items_ids: HashSet<String> =
            related_items_ids.iter().map(def_id_to_string).collect();
        let related_contracts = contracts
            .into_iter()
            .filter(|c| {
                let id = c
                    .function_tested()
                    .map(|path| path.join("::").to_string())
                    .unwrap_or("!".to_string());
                related_items_ids.contains(&id)
            })
            .collect();

        PromptContext {
            tested_item: function_tested,
            tested_item_contents,
            related_items_contents,
            related_contracts,
        }
    }
}
