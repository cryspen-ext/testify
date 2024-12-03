# Testify

Status: 
 - most of the functionality is implemented
 - most of the contract of the PoC are going through (all but those
   with contracts with universally quantified types)
 - there is no CLI, no input language
 - there are bugs
 - there is a begining of error handling, but I need to improve that
 - types are infered via hax

## General design
A contract is defined as the following (see in `lib.rs`):
```rust
pub struct Contract {
    pub inputs: Vec<Input>,
    pub description: String,
    pub precondition: syn::Expr,
    pub postcondition: syn::Expr,
    pub span: Span,
    pub dependencies: HashMap<String, String>,
    pub use_statements: Vec<syn::ItemUse>,
}
```

The global architecture resolves around a notion of pool of
contracts. A pool hosts a set of contracts that are compatible in term
of Rust dependencies.

On a pool we can:
 - `pool.instantiate_types()`: instantiates generic types (for now this is the identity);
 - `pool.instantiate_values()`: instantiates values randomly for every input of every contract;
 - `pool.compute_eval_nodes()`: get rid of abstractions by partially computing sub expressions in every contract.
 
At the end, we get a pool of concrete contract, which we can export as assertions.

## Todos
 - A CLI frontend
 - An input language for contracts: currently, `imported.rs` defines a big vector of `Contract`s via a macro.
 - Use hax to get type definitions, so that we can generate printers and marshalling automatically

## Test
To test this, you need to make sure you have hax installed and in
path. `cargo run` will produce `assertions.rs`.
