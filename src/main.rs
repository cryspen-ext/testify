use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use testify::prelude::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    contracts: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generates tests and check coverage
    Generate { output: PathBuf },
    /// Auto complete empty contracts
    Auto {
        #[arg(long)]
        ollama: bool,
    },
}

#[derive(fmt_derive::Debug, Clone, Serialize, Deserialize)]
struct ContractsFile {
    contracts: Vec<Contract>,
}

fn main() {
    testify::driver::setup_tracing();

    let cli = Cli::parse();
    let ContractsFile { mut contracts } =
        toml::from_str(&fs::read_to_string(&cli.contracts).unwrap()).unwrap();

    contracts.iter_mut().for_each(|c| c.normalize_paths());

    match &cli.command {
        Command::Generate { output } => testify::driver::run(contracts, output),
        Command::Auto { ollama } => {
            if *ollama {
                let mut llm = testify::llm::LLM.lock().unwrap();
                *llm = Box::new(testify::llm::Ollama {
                    endpoint: "http://localhost:11434/api".to_string(),
                    model: "qwen2.5-coder:14b".to_string(),
                });
            }
            let real_contracts: Vec<_> = contracts
                .iter()
                .cloned()
                .filter(|c| !c.is_default())
                .collect();
            let contracts: Vec<_> = contracts
                .iter()
                .cloned()
                .map(|mut contract| {
                    if let (true, Some(target)) = (contract.is_default(), &contract.function_tested)
                    {
                        let ctx = testify::llm::PromptContext::new(
                            &contract.dependencies,
                            target.clone(),
                            &real_contracts,
                        );
                        let testify::llm::PromptResult {
                            inputs,
                            postcondition,
                            precondition,
                        } = ctx.ask();

                        contract.inputs = inputs;
                        contract.precondition = precondition;
                        contract.postcondition = postcondition;
                    };
                    contract
                })
                .collect();
            let contracts = ContractsFile { contracts };
            fs::write(&cli.contracts, toml::to_string(&contracts).unwrap()).unwrap();
        }
    }
}
