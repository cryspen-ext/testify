use colored::Colorize;
use std::collections::HashMap;
use syn::parse_quote;
use testify::*;

fn require_binary(bin: &str) {
    if which::which(bin).is_err() {
        println!("{}", format!("Could not find binary {}", bin.bold()).red());
        std::process::exit(1);
    }
}

fn non_core_example_contract() -> Contract {
    Contract {
        inputs: vec![Input {
            name: "x".to_string(),
            kind: InputKind::Value {
                typ: parse_quote! {u8},
                aliases: vec![],
            },
        }],
        description: "Test `add_or_zero(x, x)`".to_string(),
        precondition: parse_quote! {x.up() + x.up() < 256u16.up()},
        postcondition: parse_quote! { example_crate::add_or_zero(x, x) == eval(u8::down(x.up() + x.up())) },
        span: Span::dummy(),
        dependencies: HashMap::from_iter(
            [
                (
                    "abstractions".to_string(),
                    format!(
                        r#"{{path = "{}/abstractions"}}"#,
                        std::env!("CARGO_MANIFEST_DIR")
                    ),
                ),
                (
                    "example-crate".to_string(),
                    format!(
                        r#"{{path = "{}/example-crate"}}"#,
                        std::env!("CARGO_MANIFEST_DIR")
                    ),
                ),
            ]
            .into_iter(),
        ),
        use_statements: vec![syn::parse_quote! {use abstractions::*;}],
        function_tested: Some(parse_quote! {example_crate::add_or_zero}),
    }
}

fn main() {
    require_binary("cargo-tarpaulin");

    let mut contracts = testify::imported::contracts();
    contracts.push(non_core_example_contract());
    let contracts_len = contracts.len();

    let pools = pool::ContractPool::new_pools(contracts);
    println!(
        "Processing {} contracts in {} pools",
        format!("{}", contracts_len).bold(),
        format!("{}", pools.len()).bold()
    );

    let outfile = "assertions.rs";
    use std::{fs, io::Write};
    fs::remove_file(outfile);

    let mut resulting_assertions = vec![];

    for (nth, pool) in pools.into_iter().enumerate() {
        println!(" ① Instantiating types (pool {})...", nth + 1);
        let pool = pool.instantiate_types();
        println!(" ② Instantiating values (pool {})...", nth + 1);
        let mut pool = pool.instantiate_values();
        println!(" ③ Computing eval nodes (pool {})...", nth + 1);
        pool.compute_eval_nodes();
        println!(" ④ Computing coverage (pool {})...", nth + 1);
        pool.compute_coverage();
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
