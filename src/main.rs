use testify::*;

// mod assertions;

fn main() {
    let mut contracts = testify::imported::contracts();

    let contracts = contracts
        .into_iter()
        // .filter(|x| &x.description == "Semantics of the saturating division by non-zero")
        .collect::<Vec<_>>();

    println!("Processing {} contracts", contracts.len());

    for pool in pool::ContractPool::new_pools(contracts) {
        println!("Instantiating types...");
        let pool = pool.instantiate_types();
        println!("Instantiating values...");
        let mut pool = pool.instantiate_values();
        println!("Computing eval nodes...");
        pool.compute_eval_nodes();

        let assertions = pool
            .contracts()
            .iter()
            .map(|contract| contract.as_assertion());
        let assertions = prettyplease::unparse(&syn::parse_quote! {
            fn main() {
                #(#assertions)*
            }
        });

        std::fs::write("assertions.rs", assertions).expect("Unable to write file");
    }

    println!("Hello, world!");
}
