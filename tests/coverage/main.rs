use std::collections::HashMap;
use syn::parse_quote;
use testify::{Contract, Input, InputKind, Span};

fn main() {
    testify::driver::run(
        vec![Contract {
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
                            r#"{{path = "{}/tests/coverage/example-crate"}}"#,
                            std::env!("CARGO_MANIFEST_DIR")
                        ),
                    ),
                ]
                .into_iter(),
            ),
            use_statements: vec![syn::parse_quote! {use abstractions::*;}],
            function_tested: Some(parse_quote! {example_crate::add_or_zero}),
        }],
        "regressions.rs",
    );
}
