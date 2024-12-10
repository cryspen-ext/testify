use crate::prelude::*;
use crate::*;
use colored::Colorize;

/// Make sure a binary is in PATH.
fn require_binary(bin: &str) {
    if which::which(bin).is_err() {
        println!("{}", format!("Could not find binary {}", bin.bold()).red());
        std::process::exit(1);
    }
}

pub fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

/// Run the default "driver" for a list of contracts.
pub fn run(contracts: Vec<Contract>, outfile: impl AsRef<Path>) {
    require_binary("cargo-tarpaulin");

    let contracts_len = contracts.len();

    let pools = pool::ContractPool::new_pools(contracts);
    println!(
        "Processing {} contracts in {} pools",
        format!("{}", contracts_len).bold(),
        format!("{}", pools.len()).bold()
    );

    let outfile = "assertions.rs";
    use std::fs;
    fs::remove_file(outfile);

    let mut resulting_assertions = vec![];
    let mut coverage_reports = vec![];

    for (nth, pool) in pools.into_iter().enumerate() {
        println!(" ① Instantiating types (pool {})...", nth + 1);
        let pool = pool.instantiate_types();
        println!(" ② Instantiating values (pool {})...", nth + 1);
        let mut pool = pool.instantiate_values();
        println!(" ③ Computing eval nodes (pool {})...", nth + 1);
        pool.compute_eval_nodes();
        println!(" ④ Computing coverage (pool {})...", nth + 1);
        coverage_reports.extend(pool.compute_coverage());
        println!(" ⑤ Done! Saving assertions (pool {}).", nth + 1);

        let assertions = pool
            .contracts()
            .iter()
            .map(|contract| contract.as_assertion());

        resulting_assertions.extend(assertions);
    }

    fs::write(
        outfile,
        prettyplease::unparse(&syn::parse_quote! {
            fn main() {
                #(#resulting_assertions)*
            }
        }),
    )
    .expect("Unable to write file");
}
