use crate::prelude::*;
use crate::{default_tests_number, krate::Krate};

/// Represents the context needed to generate a prompt for contract generation.
/// This includes the item being tested, its contents, related items, and related contracts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptContext {
    /// The name of the item being tested.
    pub tested_item: String,
    /// The source code or definition of the item being tested.
    pub tested_item_contents: String,
    /// Contents of items related to the tested item.
    pub related_items_contents: Vec<String>,
    /// Contracts related to the tested item.
    pub related_contracts: Vec<Contract>,
}

impl Contract {
    /// Creates an empty, dummy contract with default values.
    ///
    /// This is useful for initializing a contract before populating its fields via LLM.
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
            tests: default_tests_number(),
        }
    }
}

/// Represents a common API for interacting with a language model (LLM).
pub trait LlmAPI: Sync + Send {
    /// Sends a query to the language model and returns the response as a `String`.
    ///
    /// # Arguments
    ///
    /// * `s` - The query string to send to the language model.
    fn query(&self, s: &str) -> String;
}

/// A command-line-based implementation of the `LlmAPI`: asks the user interactively to ask a LLM some query, and waits for the user to paste the result.
pub struct CliLlm;

/// An implementation of `LlmAPI` for interacting with the Ollama API.
pub struct Ollama {
    /// The model name used by Ollama.
    pub model: String,
    /// The endpoint URL for the Ollama API.
    pub endpoint: String,
}

use std::sync::{LazyLock, Mutex};

/// A globally accessible, thread-safe instance of a language model API. This might be overriden by the binary if the user selects a different LLM.
pub static LLM: LazyLock<Mutex<Box<dyn LlmAPI>>> = LazyLock::new(|| Mutex::new(Box::new(CliLlm)));

impl LlmAPI for Ollama {
    /// Queries the Ollama API with the given input string.
    ///
    /// This method constructs a JSON payload and sends it to the API endpoint,
    /// then parses the streamed response, concatenating all anwser chunks into a `String`.
    fn query(&self, s: &str) -> String {
        tracing::info!("Querying Ollama...");
        use serde_json::Value;
        let mut map = serde_json::Map::new();
        map.insert("model".into(), Value::String(self.model.clone()));
        map.insert("prompt".into(), Value::String(s.into()));
        let payload = Value::Object(map);
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&format!("{}/generate", self.endpoint))
            .json(&payload)
            .send()
            .unwrap();

        use serde_jsonlines::BufReadExt;
        use std::io::BufReader;
        let reader = BufReader::new(res);
        let mut out = String::new();
        for response in reader.json_lines::<serde_json::Value>() {
            out += response.unwrap()["response"].as_str().unwrap();
        }
        tracing::info!("Got output `{out}`");
        out
    }
}

impl LlmAPI for CliLlm {
    /// Prompts the user to query a language model manually and returns the result.
    ///
    /// The user is expected to paste the model's response.
    fn query(&self, s: &str) -> String {
        println!("----------------------------------");
        println!("Please ask a LLM the following:");
        println!("----------------------------------");
        println!("{}\nPease end the message with `<EOF>`.", s);
        println!("----------------------------------");
        println!("Copy paste your answer here and press enter:");
        let mut s = String::new();
        loop {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            if buffer.trim() == "<EOF>" {
                break;
            }
            s += &buffer;
        }
        s
    }
}

impl ToString for PromptContext {
    /// Converts the `PromptContext` into a formatted string suitable for prompting an LLM.
    ///
    /// The string includes the tested item's definition, related contracts, and additional items of interest.
    fn to_string(&self) -> String {
        let Self {
            tested_item,
            tested_item_contents,
            related_items_contents,
            related_contracts,
        } = self;
        let related_contracts = serde_json::to_string(related_contracts).unwrap();
        let related_items_contents = related_items_contents.join("\n\n");
        format!(
            r#"
Generate a contract for the function `{tested_item}`, its definition is:
```rust
{tested_item_contents}
```

Here are other contracts that you can use to infer the contract:
```json
{related_contracts}
```

Additional functions that might be of interest:
```rust
{related_items_contents}
```

A contract is:
 - A comma-separated list of Rust inputs: e.g. `example_var:ExampleTy`.
 - A precondition, as a Rust boolean expression: e.g. `3 + 3 == 6`.
 - A postcondition, as a Rust boolean expression that calls the function `{tested_item}` with the inputs.

Please output only those three items, without any english text.
Please separate each with `--------------`.
"#
        )
    }
}

/// Represents the result of a prompt sent to the LLM, containing extracted inputs, precondition, and postcondition.
pub struct PromptResult {
    /// The inputs extracted from the prompt result.
    pub inputs: Vec<Input>,
    /// The precondition as a Rust expression.
    pub precondition: syn::Expr,
    /// The postcondition as a Rust expression.
    pub postcondition: syn::Expr,
}

impl PromptContext {
    /// Sends the current context as a prompt to the LLM and parses the result into a `PromptResult`.
    pub fn ask(&self) -> PromptResult {
        let prompt = self.to_string();
        let output = LLM.lock().unwrap().query(&prompt);
        let lines: Vec<_> = output.split("--------------").map(|s| s.trim()).collect();
        let [inputs, precondition, postcondition] = &lines[..] else {
            eprintln!("Wrong answer from LLM: <{}>. Quitting.", output);
            std::process::exit(1);
        };
        use std::str::FromStr;
        let inputs = inputs
            .split(",")
            .map(|input| input.split_once(":").unwrap())
            .map(|(name, typ)| Input {
                name: name.to_string(),
                kind: InputKind::Value {
                    typ: {
                        let ts = proc_macro2::TokenStream::from_str(typ).unwrap();
                        syn::parse2(ts).unwrap()
                    },
                    aliases: vec![],
                },
            })
            .collect::<Vec<_>>();
        let precondition =
            proc_macro2::TokenStream::from_str(precondition).expect("Wrong anwser from LLM");
        let precondition = syn::parse2(precondition).expect("Wrong anwser from LLM");
        let postcondition =
            proc_macro2::TokenStream::from_str(postcondition).expect("Wrong anwser from LLM");
        let postcondition = syn::parse2(postcondition).expect("Wrong anwser from LLM");
        PromptResult {
            precondition,
            postcondition,
            inputs,
        }
    }

    /// Constructs a new `PromptContext` using the provided dependencies, item to test, and contracts.
    pub fn new(
        dependencies: &HashMap<String, DependencySpec>,
        item_to_test: syn::Path,
        contracts: &Vec<Contract>,
    ) -> PromptContext {
        let mut contract = Contract::dummy();
        contract.dependencies = dependencies.clone();
        contract.function_tested = Some(item_to_test);
        let function_tested = contract.function_tested().unwrap();
        let krate_name = &function_tested[0];
        let krate = {
            // Find the full path to the source of the crate `krate_name`.
            let krate_path = {
                // `krate` is a dummy crate whose dependencies
                // are matching dependencies declared in the
                // contracts `contracts`
                let krate = {
                    let mut krate = Krate::new();
                    krate.add_dependencies(&contract.dependencies);
                    krate
                };
                // Runs `cargo metadata`, and finds the path
                // to the `Cargo.toml` of the crate `krate_name`.
                // println!("manifest_path={manifest_path}");
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
            .cloned()
            .collect();

        PromptContext {
            tested_item: function_tested,
            tested_item_contents,
            related_items_contents,
            related_contracts,
        }
    }
}
