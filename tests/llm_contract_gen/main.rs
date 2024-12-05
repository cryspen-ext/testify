use std::collections::HashMap;
use syn::parse_quote;
use testify::{Contract, Input, InputKind, Span};

fn main() {
    testify::driver::setup_tracing();

    let deps = HashMap::from_iter(
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
                    r#"{{path = "{}/tests/llm_contract_gen/example-crate"}}"#,
                    std::env!("CARGO_MANIFEST_DIR")
                ),
            ),
        ]
        .into_iter(),
    );
    let increment_tot = Contract {
        inputs: vec![Input {
            name: "x".to_string(),
            kind: InputKind::Value {
                typ: parse_quote! {u8},
                aliases: vec![],
            },
        }],
        description: "Non panicking semantics for `increment`".to_string(),
        precondition: parse_quote! {x.up() < 255u16.up()},
        postcondition: parse_quote! { example_crate::increment(x) == x + 1 },
        span: Span::dummy(),
        dependencies: deps.clone(),
        use_statements: vec![syn::parse_quote! {use abstractions::*;}],
        function_tested: Some(parse_quote! {example_crate::increment}),
    };
    let increment_panics = Contract {
        inputs: vec![Input {
            name: "x".to_string(),
            kind: InputKind::Value {
                typ: parse_quote! {u8},
                aliases: vec![],
            },
        }],
        description: "Non panicking semantics for `increment`".to_string(),
        precondition: parse_quote! {x.up() == 255u16.up()},
        postcondition: parse_quote! { panics!(example_crate::increment(x)) },
        span: Span::dummy(),
        dependencies: deps.clone(),
        use_statements: vec![syn::parse_quote! {use abstractions::*;}],
        function_tested: Some(parse_quote! {example_crate::increment}),
    };

    let ctx = testify::llm::PromptContext::new(
        deps.clone(),
        parse_quote! {example_crate::double_increment},
        vec![increment_panics, increment_tot],
    );
    println!("ctx={}", serde_json::to_string_pretty(&ctx).unwrap());
}
