# Testify

Status: 
 - most of the functionality is implemented
 - most of the contract of the PoC are going through (all but those
   with contracts with universally quantified types)
 - there is no CLI, no input language
 - there are bugs
 - there is a begining of error handling, but I need to improve that
 - types are infered via hax

## Prerequisite
 - `tarpaulin`: `cargo install tarpaulin`

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

## Coverage

Testify provides a mechanism to check the code coverage for functions tested by contracts. Each contract may include an optional field, `function_tested`, indicating which function it is intended to test. When this field is set, **Testify** will verify the coverage of that specific function to ensure the contract exercises all its branches and paths effectively.

To achieve this, the tool works as follows:

1. **Locate the Crate**: For each function to be tested, **Testify** finds the corresponding crate using `cargo metadata`, ensuring all dependencies match those required by the contracts.
2. **Duplicate and Prepare the Crate**: The identified crate is duplicated and renamed in a temporary location to facilitate modifications.
3. **Identify Function Span**: Using the `hax` tool, **Testify** determines the precise location (span) of the function within the source code.
4. **Generate Test Function**: A test function, `testify_test`, is generated based on the assertions derived from the contracts and inserted directly after the target function within the duplicated crate.
5. **Format Code**: The duplicated crate is formatted using `rustfmt` to improve readability and ensure accurate per-line analysis.
6. **Run Tarpaulin**: The modified crate is then analyzed using `cargo-tarpaulin` to determine coverage, specifically focusing on the span of the function being tested.
7. **Generate Coverage Report**: Any lines not covered by the tests are reported, and a snippet of the source code highlights these uncovered lines for better insight.

This mechanism provides a clear and actionable report on any untested code, guiding developers to write more comprehensive contracts and increase confidence in the correctness of their implementations.

## Demo

You can run the two following demos, that are hardcoding examples:

- `cargo run --bin test-coverage`
- `cargo run --bin test-libcore-legacy-contracts`


## Todos
 - A CLI frontend
 - An input language for contracts: currently, `imported.rs` defines a big vector of `Contract`s via a macro.
 - Use hax to get type definitions, so that we can generate printers and marshalling automatically

## Test
To test this, you need to make sure you have hax installed and in
path. `cargo run` will produce `assertions.rs`.
