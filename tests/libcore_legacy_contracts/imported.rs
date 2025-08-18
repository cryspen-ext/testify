use testify::{Contract, Input, InputKind, Span};

macro_rules! contract {
    {
        header: $header:expr,
        inputs: $(<$tinput:ident>)?[$($input:ident : $input_ty:ty),*],
        precondition: $pre_body: expr,
        postcondition: $post_body:expr
            $(,test_vector: [$($test_vector:expr),*])?
            $(,strategy: $strategy:ident)?
            $(,n: $n:literal)?
            $(,n_min: $n_min:literal)?
            $(,)?
    } => {
        {
            Some(Contract {
                description: $header.to_string(),
                inputs: vec![
                    $(
                        Input {
                            name: stringify!($tinput).to_string(),
                            kind: InputKind::Type {
                                bounds: syn::parse_quote!{where},
                            }
                        },
                    )?
                        $(
                            Input {
                                name: stringify!($input).to_string(),
                                kind: InputKind::Value {
                                    typ: syn::parse_quote!{$input_ty},
                                    aliases: vec![],
                                }
                            },
                        )*
                ],
                precondition: syn::parse_quote!{$pre_body},
                postcondition: syn::parse_quote!{$post_body},
                span: Span::dummy(),
                seed: None,
                tests: 5,
                dependencies: toml::from_str(&format!(
                r#"
abstractions = {{path = "{}/abstractions"}}
"#,
                std::env!("CARGO_MANIFEST_DIR"),
            ))
            .unwrap(),
                use_statements: vec![syn::parse_quote!{abstractions::*}],
                function_tested: None,
            })
        }
    };
}

pub fn contracts() -> Vec<Contract> {
    vec![
        // contract! {
        //     header : "`is_some` is a shorthand to pattern matching", inputs : < T >
        //         [v : Option < T >], precondition : true, postcondition : v.is_some() ==
        //         (match v { Some(_) => true, None => false }),
        // },
        // contract! {
        //     header : "`is_none` is a shorthand to pattern matching", inputs : < T >
        //         [v : Option < T >], precondition : true, postcondition : v.is_none() ==
        //         (match v { Some(_) => false, None => true }),
        // },
        // contract! {
        //     header : "Unwrapping a [`None`] with `expect` always panic", inputs : < T >
        //         [v : Option < T >], precondition : v.is_none(), postcondition : panics!
        //         (v.expect("message")), n : 1,
        // },
        // contract! {
        //     header : "Unwrapping a [`Some(_)`] with `expect` always succeeds", inputs : <
        //         T > [v : Option < T >], precondition : v.is_some(), postcondition :
        //     doesn_t_panic! (v.expect("message")),
        // },
        // contract! {
        //     header : "Wrapping a value in a `Some` and unwrapping is identity", inputs : <
        //         T > [v : T], precondition : true, postcondition : Some(v).unwrap() == v,
        // },
        // contract! {
        //     header :
        //     "Applying `f` on `Some(v)` via `map` is equal to wrapping in `Some` the application of `v` to `f`",
        //     inputs : < T > [v : T, f : Fn1 < T, T >], precondition : true, postcondition :
        //     Some(v).map(f) == Some((f) (v)),
        // },
        // contract! {
        //     header : "Mapping a `None` is the identity", inputs : < T >
        //         [v : Option < T >, f : Fn1 < T, T >], precondition : v.is_none(),
        //     postcondition : v.map(f) == v, n : 1,
        // },
        // contract! {
        //     header :
        //     "The filtering of `Some(v)` with a predicate `f` being non-empty is equivalent to applying a predicate `f` on `v`",
        //     inputs : < T > [v : T, f : FnR1 < T, bool >], precondition : true,
        //     postcondition : Some(v).filter(f).is_some() == f(& v),
        // },
        // contract! {
        //     header : "Filtering a `None` is the identity", inputs : < T >
        //         [v : Option < T >, f : FnR1 < T, bool >], precondition : v.is_none(),
        //     postcondition : v.filter(f) == v, n : 1,
        // },
        // contract! {
        //     header : "Nested `Some`s", inputs : < T > [x : T], precondition : true,
        //     postcondition : Some(Some(x)).flatten() == Some(x),
        // },
        // contract! {
        //     header : "Nested or direct `None` flattens to None", inputs : < T >
        //         [x : Option < Option < T >>], precondition : x.is_none() ||
        //         x.unwrap().is_none(), postcondition : x.flatten() == None, n : 2,
        // },
        // contract! {
        //     header : "Take steals a value", inputs : < T > [x : Option < T >],
        //     precondition : true, postcondition :
        //     { let mut y = x.clone() ; y.take() == x && y.is_none() },
        // },
        // contract! {
        //     header : "Zipping two non-empty options", inputs : < T > [x : T, y : T],
        //     precondition : true, postcondition : Some(x).zip(Some(y)) == Some((x, y)),
        // },
        // contract! {
        //     header : "Zipping two options when one is a `None` makes `None`", inputs : < T
        //         > [x : Option < T >, y : Option < T >], precondition : x.is_none() ||
        //         y.is_none(), postcondition : x.zip(y).is_none(),
        // },
        // contract! {
        //     header : "Unwrapping a [`None`] always panic", inputs : < T >
        //         [v : Option < T >], precondition : v.is_none(), postcondition : panics!
        //         (v.unwrap()), n : 1,
        // },
        // contract! {
        //     header : "Unwrapping a [`Some(_)`] always succeeds", inputs : < T >
        //         [v : Option < T >], precondition : v.is_some(), postcondition : doesn_t_panic!
        //         (v.unwrap()),
        // },
        // contract! {
        //     header : "In place update via `as_mut` is equivalent to functional update",
        //     inputs : [v : Option < u8 >], precondition : v.is_some() && v.unwrap() < 50,
        //     postcondition :
        //     {
        //         let(v_unwrapped, mut v_mut) = (v.unwrap().clone(), v) ; *
        //             v_mut.as_mut().unwrap() += 10 ; v_mut.unwrap() == v_unwrapped + 10
        //     },
        // },
        // contract! {
        //     header : "[`None.as_slice()`] should always result in an empty slice", inputs
        //         : < T > [v : Option < T >], precondition : v.is_none(), postcondition :
        //     { v.as_slice().is_empty() }, n : 1
        // },
        // contract! {
        //     header :
        //     "[`Some(v).as_slice()`] should always result in a slice containing exactly `v`",
        //     inputs : < T > [v : T], precondition : true, postcondition :
        //     { Some(v).as_slice() == [v] },
        // },
        // contract! {
        //     header : "Indexing", inputs : [v : Vec < u8 >, i : usize], precondition :
        //     v.len() > 0, postcondition : v.get(eval(i % v.len())) ==
        //         Some(& eval(v [i % v.len()])),
        // },
        contract! {
            header : "Semantics of rem", inputs : [x : u8, y : u8], precondition : y != 0,
            postcondition : x % y == eval(u8 :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u8], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u8, y : u8], precondition :
            y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(u8 :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u8], precondition : true,
            postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : u8],
            precondition : x == u8 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(u8 :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : u8], precondition : x != u8
                :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition : x
                << y == eval(u8 :: down((x.up() << y) & u8 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : panics! (x << y), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition :
            x.checked_shl(y) == Some(eval(u8 :: down((x.up() << y) & u8 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : x.checked_shl(y) == None,
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u8 :: down((x.up() << y) & u8 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : x.overflowing_shl(y) ==
                (eval(u8 :: down((x.up() << (y & (u8 :: BITS - 1)) & u8 :: MAX.up()))), true),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition : x
                >> y == eval(u8 :: down((x.up() >> y) & u8 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : panics! (x >> y), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition :
            x.checked_shr(y) == Some(eval(u8 :: down((x.up() >> y) & u8 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : x.checked_shr(y) == None,
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u8, y : u32], precondition : y < u8 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u8 :: down((x.up() >> y) & u8 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs : [x : u8, y : u32],
            precondition : y >= u8 :: BITS, postcondition : x.overflowing_shr(y) ==
                (eval(u8 :: down((x.up() >> (y & (u8 :: BITS - 1)) & u8 :: MAX.up()))), true),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs : [x : u8, y : u8],
            precondition : y != 0, postcondition : x / y ==
                eval(u8 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : u8], precondition
                : true, postcondition : { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : u8, y : u8], precondition : y != 0, postcondition : x.saturating_div(y)
                == eval(u8 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : u8],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : u8, y : u8], precondition : y != 0, postcondition : x.checked_div(y) ==
                Some(eval(u8 :: down(x.up() / y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : u8],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : u8, y : u8], precondition : x.up() * y.up() <= u8 :: MAX.up(),
            postcondition : x * y == eval(u8 :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u8, y : u8], precondition :
            x.up() * y.up() > u8 :: MAX.up(), postcondition : panics! (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u8], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u8, y : u8], precondition : x.up() *
                y.up() <= u8 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u8, y : u8, z : u8], precondition :
            (x.up() * y.up() * z.up() <= u8 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u8, y : u8, z : u8], precondition :
            (x.up() * (y.up() + z.up()) <= u8 :: MAX.up() && x > 0), postcondition : x *
                (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u8], precondition : true, postcondition : x * 0
                == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : u8, y : u8], precondition : x.up() * y.up() <= u8 :: MAX.up(),
            postcondition : x.checked_mul(y) == Some(eval(u8 :: down(x.up() * y.up()))),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : u8, y : u8], precondition : x.up() * y.up() > u8 :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : u8, y : u8], precondition : x.up() * y.up() <= u8 :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(u8 :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : u8, y : u8], precondition : x.up() * y.up() > u8 :: MAX.up(),
            postcondition : x.overflowing_mul(y) == (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : u8, y : u8], precondition : true, postcondition : x.saturating_mul(y) ==
                eval(u8 :: down((x.up() * y.up()).min(u8 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : u8, y : u8], precondition : x.up() * y.up() <= u8 :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(u8 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : u8, y : u8], precondition : x.up() * y.up() > u8 :: MAX.up(),
            postcondition : x.saturating_mul(y) == u8 :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : u8, y : u8], precondition : true, postcondition : x.wrapping_mul(y) ==
                eval(u8 :: down((x.up() * y.up()) % (u8 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : u8, y : u8], precondition : x.up() * y.up() <= u8 :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(u8 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u8], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u8, y : u8], precondition : x.up() *
                y.up() <= u8 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u8, y : u8, z : u8], precondition :
            (x.up() * y.up() * z.up() <= u8 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u8, y : u8, z : u8], precondition :
            (x.up() * (y.up() + z.up()) <= u8 :: MAX.up() && x > 0), postcondition : x *
                (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u8], precondition : true, postcondition : x * 0
                == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() >= 0u8.up(), postcondition :
            x - y == eval(u8 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : u8, y : u8], precondition :
            x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs : [x : u8, y : u8],
            precondition : x.up() - y.up() >= 0u8.up(), postcondition : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() >= 0u8.up(), postcondition :
            x.wrapping_sub(y) == eval(u8 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.wrapping_sub(y) == eval(u8 :: down(x.up() - y.up() + u8 :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : u8, y : u8],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(u8 :: down((x.up() - y.up()).rem_euclid(& (u8 :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : u8, y : u8], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() >= 0u8.up(), postcondition :
            x.checked_sub(y) == Some(eval(u8 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() >= 0u8.up(), postcondition :
            (x.checked_sub(y)).and_then(| r | r.checked_add(y)) == Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs : [x : u8, y : u8],
            precondition : true, postcondition : x.saturating_sub(y) ==
                eval(u8 :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() >= 0u8.up(), postcondition :
            x.saturating_sub(y) == eval(u8 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : u8, y : u8], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : u8, y : u8],
            precondition : x.up() + y.up() <= u8 :: MAX.up(), postcondition : x + y ==
                eval(u8 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u8, y : u8], precondition :
            x.up() + y.up() > u8 :: MAX.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : u8, y : u8], precondition : x.up() +
                y.up() <= u8 :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u8], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : u8, y : u8, z : u8], precondition :
            x.up() + y.up() + z.up() <= u8 :: MAX.up(), postcondition : (x + y) + z == x +
                (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs : [x : u8, y : u8],
            precondition : true, postcondition : x.wrapping_add(y) ==
                eval(u8 :: down((x.up() + y.up()) % (u8 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : u8, y : u8], precondition : x.up() + y.up() <= u8 :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(u8 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : u8, y : u8], precondition : x.up() + y.up() > u8 :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(u8 :: down(x.up() + y.up() - u8 :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : u8, y : u8], precondition : true,
            postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u8], precondition : true,
            postcondition : { let zero : u8 = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : u8, y : u8, z : u8], precondition :
            x.up() + y.up() + z.up() <= u8 :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : u8, y : u8], precondition : x.up() + y.up() <= u8 :: MAX.up(),
            postcondition : x.checked_add(y) == Some(eval(u8 :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : u8, y : u8], precondition :
            x.up() + y.up() > u8 :: MAX.up(), postcondition : x.checked_add(y) == None,
        },
        contract! {
            header : "Commutativity", inputs : [x : u8, y : u8], precondition : true,
            postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u8], precondition : true,
            postcondition : x.checked_add(0u8) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : u8], precondition : true,
            postcondition : 0u8.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : u8, y : u8, z : u8], precondition :
            true, postcondition : x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs : [x : u8, y : u8],
            precondition : true, postcondition : x.saturating_add(y) ==
                eval(u8 :: down((x.up() + y.up()).min(u8 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : u8, y : u8], precondition : x.up() + y.up() <= u8 :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(u8 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : u8, y : u8], precondition : x.up() + y.up() > u8 :: MAX.up(),
            postcondition : x.saturating_add(y) == u8 :: MAX,
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u16, y : u16], precondition : y !=
                0, postcondition : x % y == eval(u16 :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u16], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u16, y : u16], precondition
                : y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(u16 :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u16], precondition : true,
            postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : u16],
            precondition : x == u16 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(u16 :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : u16], precondition : x !=
                u16 :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition : x
                << y == eval(u16 :: down((x.up() << y) & u16 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u16, y : u32],
            precondition : y >= u16 :: BITS, postcondition : panics! (x << y), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition :
            x.checked_shl(y) == Some(eval(u16 :: down((x.up() << y) & u16 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u16, y : u32],
            precondition : y >= u16 :: BITS, postcondition : x.checked_shl(y) == None,
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u16 :: down((x.up() << y) & u16 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u16, y : u32],
            precondition : y >= u16 :: BITS, postcondition : x.overflowing_shl(y) ==
                (eval(u16 :: down((x.up() << (y & (u16 :: BITS - 1)) & u16 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition : x
                >> y == eval(u16 :: down((x.up() >> y) & u16 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u16, y : u32], precondition : y >= u16 :: BITS, postcondition : panics!
                (x >> y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition :
            x.checked_shr(y) == Some(eval(u16 :: down((x.up() >> y) & u16 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u16, y : u32], precondition : y >= u16 :: BITS, postcondition :
            x.checked_shr(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u16, y : u32], precondition : y < u16 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u16 :: down((x.up() >> y) & u16 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u16, y : u32], precondition : y >= u16 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u16 :: down((x.up() >> (y & (u16 :: BITS - 1)) & u16 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs : [x : u16, y : u16],
            precondition : y != 0, postcondition : x / y ==
                eval(u16 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : u16], precondition
                : true, postcondition : { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : u16, y : u16], precondition : y != 0, postcondition : x.saturating_div(y)
                == eval(u16 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : u16],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : u16, y : u16], precondition : y != 0, postcondition : x.checked_div(y) ==
                Some(eval(u16 :: down(x.up() / y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : u16],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : u16, y : u16], precondition : x.up() * y.up() <= u16 :: MAX.up(),
            postcondition : x * y == eval(u16 :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u16, y : u16], precondition
                : x.up() * y.up() > u16 :: MAX.up(), postcondition : panics! (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u16], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u16, y : u16], precondition : x.up() *
                y.up() <= u16 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u16, y : u16, z : u16], precondition :
            (x.up() * y.up() * z.up() <= u16 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u16, y : u16, z : u16], precondition
                : (x.up() * (y.up() + z.up()) <= u16 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u16], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : u16, y : u16], precondition : x.up() * y.up() <= u16 :: MAX.up(),
            postcondition : x.checked_mul(y) == Some(eval(u16 :: down(x.up() * y.up()))),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : u16, y : u16], precondition : x.up() * y.up() > u16 :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : u16, y : u16], precondition : x.up() * y.up() <= u16 :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(u16 :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : u16, y : u16], precondition : x.up() * y.up() > u16 :: MAX.up(),
            postcondition : x.overflowing_mul(y) == (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : u16, y : u16], precondition : true, postcondition : x.saturating_mul(y)
                == eval(u16 :: down((x.up() * y.up()).min(u16 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : u16, y : u16], precondition : x.up() * y.up() <= u16 :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(u16 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : u16, y : u16], precondition : x.up() * y.up() > u16 :: MAX.up(),
            postcondition : x.saturating_mul(y) == u16 :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : u16, y : u16], precondition : true, postcondition : x.wrapping_mul(y) ==
                eval(u16 :: down((x.up() * y.up()) % (u16 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : u16, y : u16], precondition : x.up() * y.up() <= u16 :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(u16 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u16], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u16, y : u16], precondition : x.up() *
                y.up() <= u16 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u16, y : u16, z : u16], precondition :
            (x.up() * y.up() * z.up() <= u16 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u16, y : u16, z : u16], precondition
                : (x.up() * (y.up() + z.up()) <= u16 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u16], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x - y == eval(u16 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : u16, y : u16], precondition
                : x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.wrapping_sub(y) == eval(u16 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.wrapping_sub(y) == eval(u16 :: down(x.up() - y.up() + u16 :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : u16, y : u16],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(u16 :: down((x.up() - y.up()).rem_euclid(& (u16 :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : u16, y : u16], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.checked_sub(y) == Some(eval(u16 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x.checked_sub(y)).and_then(| r | r.checked_add(y)) == Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs :
            [x : u16, y : u16], precondition : true, postcondition : x.saturating_sub(y)
                == eval(u16 :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.saturating_sub(y) == eval(u16 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : u16, y : u16], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : u16, y : u16],
            precondition : x.up() + y.up() <= u16 :: MAX.up(), postcondition : x + y ==
                eval(u16 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u16, y : u16], precondition
                : x.up() + y.up() > u16 :: MAX.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : u16, y : u16], precondition : x.up() +
                y.up() <= u16 :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u16], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : u16, y : u16, z : u16], precondition :
            x.up() + y.up() + z.up() <= u16 :: MAX.up(), postcondition : (x + y) + z == x
                + (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs : [x : u16, y : u16],
            precondition : true, postcondition : x.wrapping_add(y) ==
                eval(u16 :: down((x.up() + y.up()) % (u16 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : u16, y : u16], precondition : x.up() + y.up() <= u16 :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(u16 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : u16, y : u16], precondition : x.up() + y.up() > u16 :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(u16 :: down(x.up() + y.up() - u16 :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : u16, y : u16], precondition : true,
            postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u16], precondition : true,
            postcondition : { let zero : u16 = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : u16, y : u16, z : u16], precondition :
            x.up() + y.up() + z.up() <= u16 :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : u16, y : u16], precondition : x.up() + y.up() <= u16 :: MAX.up(),
            postcondition : x.checked_add(y) == Some(eval(u16 :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : u16, y : u16], precondition :
            x.up() + y.up() > u16 :: MAX.up(), postcondition : x.checked_add(y) == None,
        },
        contract! {
            header : "Commutativity", inputs : [x : u16, y : u16], precondition : true,
            postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u16], precondition : true,
            postcondition : x.checked_add(0u16) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : u16], precondition : true,
            postcondition : 0u16.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : u16, y : u16, z : u16], precondition :
            true, postcondition : x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs : [x : u16, y : u16],
            precondition : true, postcondition : x.saturating_add(y) ==
                eval(u16 :: down((x.up() + y.up()).min(u16 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : u16, y : u16], precondition : x.up() + y.up() <= u16 :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(u16 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : u16, y : u16], precondition : x.up() + y.up() > u16 :: MAX.up(),
            postcondition : x.saturating_add(y) == u16 :: MAX,
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u32, y : u32], precondition : y !=
                0, postcondition : x % y == eval(u32 :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u32], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u32, y : u32], precondition
                : y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(u32 :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u32], precondition : true,
            postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : u32],
            precondition : x == u32 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(u32 :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : u32], precondition : x !=
                u32 :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition : x
                << y == eval(u32 :: down((x.up() << y) & u32 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u32, y : u32],
            precondition : y >= u32 :: BITS, postcondition : panics! (x << y), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition :
            x.checked_shl(y) == Some(eval(u32 :: down((x.up() << y) & u32 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u32, y : u32],
            precondition : y >= u32 :: BITS, postcondition : x.checked_shl(y) == None,
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u32 :: down((x.up() << y) & u32 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u32, y : u32],
            precondition : y >= u32 :: BITS, postcondition : x.overflowing_shl(y) ==
                (eval(u32 :: down((x.up() << (y & (u32 :: BITS - 1)) & u32 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition : x
                >> y == eval(u32 :: down((x.up() >> y) & u32 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u32, y : u32], precondition : y >= u32 :: BITS, postcondition : panics!
                (x >> y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition :
            x.checked_shr(y) == Some(eval(u32 :: down((x.up() >> y) & u32 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u32, y : u32], precondition : y >= u32 :: BITS, postcondition :
            x.checked_shr(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u32, y : u32], precondition : y < u32 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u32 :: down((x.up() >> y) & u32 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u32, y : u32], precondition : y >= u32 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u32 :: down((x.up() >> (y & (u32 :: BITS - 1)) & u32 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs : [x : u32, y : u32],
            precondition : y != 0, postcondition : x / y ==
                eval(u32 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : u32], precondition
                : true, postcondition : { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : u32, y : u32], precondition : y != 0, postcondition : x.saturating_div(y)
                == eval(u32 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : u32],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : u32, y : u32], precondition : y != 0, postcondition : x.checked_div(y) ==
                Some(eval(u32 :: down(x.up() / y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : u32],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : u32, y : u32], precondition : x.up() * y.up() <= u32 :: MAX.up(),
            postcondition : x * y == eval(u32 :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u32, y : u32], precondition
                : x.up() * y.up() > u32 :: MAX.up(), postcondition : panics! (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u32], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u32, y : u32], precondition : x.up() *
                y.up() <= u32 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u32, y : u32, z : u32], precondition :
            (x.up() * y.up() * z.up() <= u32 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u32, y : u32, z : u32], precondition
                : (x.up() * (y.up() + z.up()) <= u32 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u32], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : u32, y : u32], precondition : x.up() * y.up() <= u32 :: MAX.up(),
            postcondition : x.checked_mul(y) == Some(eval(u32 :: down(x.up() * y.up()))),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : u32, y : u32], precondition : x.up() * y.up() > u32 :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : u32, y : u32], precondition : x.up() * y.up() <= u32 :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(u32 :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : u32, y : u32], precondition : x.up() * y.up() > u32 :: MAX.up(),
            postcondition : x.overflowing_mul(y) == (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : u32, y : u32], precondition : true, postcondition : x.saturating_mul(y)
                == eval(u32 :: down((x.up() * y.up()).min(u32 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : u32, y : u32], precondition : x.up() * y.up() <= u32 :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(u32 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : u32, y : u32], precondition : x.up() * y.up() > u32 :: MAX.up(),
            postcondition : x.saturating_mul(y) == u32 :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : u32, y : u32], precondition : true, postcondition : x.wrapping_mul(y) ==
                eval(u32 :: down((x.up() * y.up()) % (u32 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : u32, y : u32], precondition : x.up() * y.up() <= u32 :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(u32 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u32], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u32, y : u32], precondition : x.up() *
                y.up() <= u32 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u32, y : u32, z : u32], precondition :
            (x.up() * y.up() * z.up() <= u32 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u32, y : u32, z : u32], precondition
                : (x.up() * (y.up() + z.up()) <= u32 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u32], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x - y == eval(u32 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : u32, y : u32], precondition
                : x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.wrapping_sub(y) == eval(u32 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.wrapping_sub(y) == eval(u32 :: down(x.up() - y.up() + u32 :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : u32, y : u32],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(u32 :: down((x.up() - y.up()).rem_euclid(& (u32 :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : u32, y : u32], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.checked_sub(y) == Some(eval(u32 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x.checked_sub(y)).and_then(| r | r.checked_add(y)) == Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs :
            [x : u32, y : u32], precondition : true, postcondition : x.saturating_sub(y)
                == eval(u32 :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.saturating_sub(y) == eval(u32 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : u32, y : u32], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : u32, y : u32],
            precondition : x.up() + y.up() <= u32 :: MAX.up(), postcondition : x + y ==
                eval(u32 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u32, y : u32], precondition
                : x.up() + y.up() > u32 :: MAX.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : u32, y : u32], precondition : x.up() +
                y.up() <= u32 :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u32], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : u32, y : u32, z : u32], precondition :
            x.up() + y.up() + z.up() <= u32 :: MAX.up(), postcondition : (x + y) + z == x
                + (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs : [x : u32, y : u32],
            precondition : true, postcondition : x.wrapping_add(y) ==
                eval(u32 :: down((x.up() + y.up()) % (u32 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : u32, y : u32], precondition : x.up() + y.up() <= u32 :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(u32 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : u32, y : u32], precondition : x.up() + y.up() > u32 :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(u32 :: down(x.up() + y.up() - u32 :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : u32, y : u32], precondition : true,
            postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u32], precondition : true,
            postcondition : { let zero : u32 = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : u32, y : u32, z : u32], precondition :
            x.up() + y.up() + z.up() <= u32 :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : u32, y : u32], precondition : x.up() + y.up() <= u32 :: MAX.up(),
            postcondition : x.checked_add(y) == Some(eval(u32 :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : u32, y : u32], precondition :
            x.up() + y.up() > u32 :: MAX.up(), postcondition : x.checked_add(y) == None,
        },
        contract! {
            header : "Commutativity", inputs : [x : u32, y : u32], precondition : true,
            postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u32], precondition : true,
            postcondition : x.checked_add(0u32) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : u32], precondition : true,
            postcondition : 0u32.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : u32, y : u32, z : u32], precondition :
            true, postcondition : x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs : [x : u32, y : u32],
            precondition : true, postcondition : x.saturating_add(y) ==
                eval(u32 :: down((x.up() + y.up()).min(u32 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : u32, y : u32], precondition : x.up() + y.up() <= u32 :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(u32 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : u32, y : u32], precondition : x.up() + y.up() > u32 :: MAX.up(),
            postcondition : x.saturating_add(y) == u32 :: MAX,
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u64, y : u64], precondition : y !=
                0, postcondition : x % y == eval(u64 :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u64], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u64, y : u64], precondition
                : y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(u64 :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u64], precondition : true,
            postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : u64],
            precondition : x == u64 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(u64 :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : u64], precondition : x !=
                u64 :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition : x
                << y == eval(u64 :: down((x.up() << y) & u64 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u64, y : u32],
            precondition : y >= u64 :: BITS, postcondition : panics! (x << y), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition :
            x.checked_shl(y) == Some(eval(u64 :: down((x.up() << y) & u64 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u64, y : u32],
            precondition : y >= u64 :: BITS, postcondition : x.checked_shl(y) == None,
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u64 :: down((x.up() << y) & u64 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs : [x : u64, y : u32],
            precondition : y >= u64 :: BITS, postcondition : x.overflowing_shl(y) ==
                (eval(u64 :: down((x.up() << (y & (u64 :: BITS - 1)) & u64 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition : x
                >> y == eval(u64 :: down((x.up() >> y) & u64 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u64, y : u32], precondition : y >= u64 :: BITS, postcondition : panics!
                (x >> y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition :
            x.checked_shr(y) == Some(eval(u64 :: down((x.up() >> y) & u64 :: MAX.up()))),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u64, y : u32], precondition : y >= u64 :: BITS, postcondition :
            x.checked_shr(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u64, y : u32], precondition : y < u64 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u64 :: down((x.up() >> y) & u64 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u64, y : u32], precondition : y >= u64 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u64 :: down((x.up() >> (y & (u64 :: BITS - 1)) & u64 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs : [x : u64, y : u64],
            precondition : y != 0, postcondition : x / y ==
                eval(u64 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : u64], precondition
                : true, postcondition : { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : u64, y : u64], precondition : y != 0, postcondition : x.saturating_div(y)
                == eval(u64 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : u64],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : u64, y : u64], precondition : y != 0, postcondition : x.checked_div(y) ==
                Some(eval(u64 :: down(x.up() / y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : u64],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : u64, y : u64], precondition : x.up() * y.up() <= u64 :: MAX.up(),
            postcondition : x * y == eval(u64 :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u64, y : u64], precondition
                : x.up() * y.up() > u64 :: MAX.up(), postcondition : panics! (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u64], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u64, y : u64], precondition : x.up() *
                y.up() <= u64 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u64, y : u64, z : u64], precondition :
            (x.up() * y.up() * z.up() <= u64 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u64, y : u64, z : u64], precondition
                : (x.up() * (y.up() + z.up()) <= u64 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u64], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : u64, y : u64], precondition : x.up() * y.up() <= u64 :: MAX.up(),
            postcondition : x.checked_mul(y) == Some(eval(u64 :: down(x.up() * y.up()))),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : u64, y : u64], precondition : x.up() * y.up() > u64 :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : u64, y : u64], precondition : x.up() * y.up() <= u64 :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(u64 :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : u64, y : u64], precondition : x.up() * y.up() > u64 :: MAX.up(),
            postcondition : x.overflowing_mul(y) == (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : u64, y : u64], precondition : true, postcondition : x.saturating_mul(y)
                == eval(u64 :: down((x.up() * y.up()).min(u64 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : u64, y : u64], precondition : x.up() * y.up() <= u64 :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(u64 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : u64, y : u64], precondition : x.up() * y.up() > u64 :: MAX.up(),
            postcondition : x.saturating_mul(y) == u64 :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : u64, y : u64], precondition : true, postcondition : x.wrapping_mul(y) ==
                eval(u64 :: down((x.up() * y.up()) % (u64 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : u64, y : u64], precondition : x.up() * y.up() <= u64 :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(u64 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u64], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u64, y : u64], precondition : x.up() *
                y.up() <= u64 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u64, y : u64, z : u64], precondition :
            (x.up() * y.up() * z.up() <= u64 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u64, y : u64, z : u64], precondition
                : (x.up() * (y.up() + z.up()) <= u64 :: MAX.up() && x > 0), postcondition : x
                * (y + z) == x * y + x * z, strategy : SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u64], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x - y == eval(u64 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : u64, y : u64], precondition
                : x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.wrapping_sub(y) == eval(u64 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.wrapping_sub(y) == eval(u64 :: down(x.up() - y.up() + u64 :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : u64, y : u64],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(u64 :: down((x.up() - y.up()).rem_euclid(& (u64 :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : u64, y : u64], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.checked_sub(y) == Some(eval(u64 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : (x.checked_sub(y)).and_then(| r | r.checked_add(y)) == Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs :
            [x : u64, y : u64], precondition : true, postcondition : x.saturating_sub(y)
                == eval(u64 :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() >= 0u8.up(), postcondition
                : x.saturating_sub(y) == eval(u64 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : u64, y : u64], precondition : x.up() - y.up() < 0u8.up(), postcondition :
            x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : u64, y : u64],
            precondition : x.up() + y.up() <= u64 :: MAX.up(), postcondition : x + y ==
                eval(u64 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u64, y : u64], precondition
                : x.up() + y.up() > u64 :: MAX.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : u64, y : u64], precondition : x.up() +
                y.up() <= u64 :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u64], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : u64, y : u64, z : u64], precondition :
            x.up() + y.up() + z.up() <= u64 :: MAX.up(), postcondition : (x + y) + z == x
                + (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs : [x : u64, y : u64],
            precondition : true, postcondition : x.wrapping_add(y) ==
                eval(u64 :: down((x.up() + y.up()) % (u64 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : u64, y : u64], precondition : x.up() + y.up() <= u64 :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(u64 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : u64, y : u64], precondition : x.up() + y.up() > u64 :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(u64 :: down(x.up() + y.up() - u64 :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : u64, y : u64], precondition : true,
            postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u64], precondition : true,
            postcondition : { let zero : u64 = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : u64, y : u64, z : u64], precondition :
            x.up() + y.up() + z.up() <= u64 :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : u64, y : u64], precondition : x.up() + y.up() <= u64 :: MAX.up(),
            postcondition : x.checked_add(y) == Some(eval(u64 :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : u64, y : u64], precondition :
            x.up() + y.up() > u64 :: MAX.up(), postcondition : x.checked_add(y) == None,
        },
        contract! {
            header : "Commutativity", inputs : [x : u64, y : u64], precondition : true,
            postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u64], precondition : true,
            postcondition : x.checked_add(0u64) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : u64], precondition : true,
            postcondition : 0u64.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : u64, y : u64, z : u64], precondition :
            true, postcondition : x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs : [x : u64, y : u64],
            precondition : true, postcondition : x.saturating_add(y) ==
                eval(u64 :: down((x.up() + y.up()).min(u64 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : u64, y : u64], precondition : x.up() + y.up() <= u64 :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(u64 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : u64, y : u64], precondition : x.up() + y.up() > u64 :: MAX.up(),
            postcondition : x.saturating_add(y) == u64 :: MAX,
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u128, y : u128], precondition : y
                != 0, postcondition : x % y == eval(u128 :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : u128], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u128, y : u128],
            precondition : y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(u128 :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : u128], precondition : true,
            postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : u128],
            precondition : x == u128 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(u128 :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : u128], precondition : x !=
                u128 :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x << y == eval(u128 :: down((x.up() << y) & u128 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition : panics!
                (x << y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x.checked_shl(y) ==
                Some(eval(u128 :: down((x.up() << y) & u128 :: MAX.up()))), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition :
            x.checked_shl(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u128 :: down((x.up() << y) & u128 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(u128 :: down((x.up() << (y & (u128 :: BITS - 1)) & u128 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x >> y == eval(u128 :: down((x.up() >> y) & u128 :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition : panics!
                (x >> y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x.checked_shr(y) ==
                Some(eval(u128 :: down((x.up() >> y) & u128 :: MAX.up()))), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition :
            x.checked_shr(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : u128, y : u32], precondition : y < u128 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u128 :: down((x.up() >> y) & u128 :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : u128, y : u32], precondition : y >= u128 :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(u128 :: down((x.up() >> (y & (u128 :: BITS - 1)) & u128 :: MAX.up()))),
                 true), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs :
            [x : u128, y : u128], precondition : y != 0, postcondition : x / y ==
                eval(u128 :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : u128],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : u128, y : u128], precondition : y != 0, postcondition :
            x.saturating_div(y) == eval(u128 :: down(x.up() / y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : u128],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : u128, y : u128], precondition : y != 0, postcondition : x.checked_div(y)
                == Some(eval(u128 :: down(x.up() / y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : u128],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : u128, y : u128], precondition : x.up() * y.up() <= u128 :: MAX.up(),
            postcondition : x * y == eval(u128 :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u128, y : u128],
            precondition : x.up() * y.up() > u128 :: MAX.up(), postcondition : panics!
                (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u128], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u128, y : u128], precondition : x.up()
                * y.up() <= u128 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u128, y : u128, z : u128],
            precondition :
            (x.up() * y.up() * z.up() <= u128 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u128, y : u128, z : u128],
            precondition : (x.up() * (y.up() + z.up()) <= u128 :: MAX.up() && x > 0),
            postcondition : x * (y + z) == x * y + x * z, strategy :
            SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u128], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : u128, y : u128], precondition : x.up() * y.up() <= u128 :: MAX.up(),
            postcondition : x.checked_mul(y) == Some(eval(u128 :: down(x.up() * y.up()))),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : u128, y : u128], precondition : x.up() * y.up() > u128 :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : u128, y : u128], precondition : x.up() * y.up() <= u128 :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(u128 :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : u128, y : u128], precondition : x.up() * y.up() > u128 ::
            MAX.up(), postcondition : x.overflowing_mul(y) ==
                (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : u128, y : u128], precondition : true, postcondition : x.saturating_mul(y)
                == eval(u128 :: down((x.up() * y.up()).min(u128 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : u128, y : u128], precondition : x.up() * y.up() <= u128 :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(u128 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : u128, y : u128], precondition : x.up() * y.up() > u128 :: MAX.up(),
            postcondition : x.saturating_mul(y) == u128 :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : u128, y : u128], precondition : true, postcondition : x.wrapping_mul(y)
                == eval(u128 :: down((x.up() * y.up()) % (u128 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : u128, y : u128], precondition : x.up() * y.up() <= u128 :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(u128 :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u128], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : u128, y : u128], precondition : x.up()
                * y.up() <= u128 :: MAX.up(), postcondition : x * y == y * x, strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : u128, y : u128, z : u128],
            precondition :
            (x.up() * y.up() * z.up() <= u128 :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : u128, y : u128, z : u128],
            precondition : (x.up() * (y.up() + z.up()) <= u128 :: MAX.up() && x > 0),
            postcondition : x * (y + z) == x * y + x * z, strategy :
            SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : u128], precondition : true, postcondition : x *
                0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x - y == eval(u128 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : u128, y : u128],
            precondition : x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.wrapping_sub(y) == eval(u128 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() < 0u8.up(), postcondition
                : x.wrapping_sub(y) == eval(u128 :: down(x.up() - y.up() + u128 :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(u128 :: down((x.up() - y.up()).rem_euclid(& (u128 :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : u128, y : u128], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.checked_sub(y) == Some(eval(u128 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() < 0u8.up(), postcondition
                : x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : (x.checked_sub(y)).and_then(| r | r.checked_add(y)) ==
                Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs :
            [x : u128, y : u128], precondition : true, postcondition : x.saturating_sub(y)
                == eval(u128 :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.saturating_sub(y) == eval(u128 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : u128, y : u128], precondition : x.up() - y.up() < 0u8.up(), postcondition
                : x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() <= u128 :: MAX.up(),
            postcondition : x + y == eval(u128 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : u128, y : u128],
            precondition : x.up() + y.up() > u128 :: MAX.up(), postcondition : panics!
                (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : u128, y : u128], precondition : x.up()
                + y.up() <= u128 :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u128], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : u128, y : u128, z : u128],
            precondition : x.up() + y.up() + z.up() <= u128 :: MAX.up(), postcondition :
            (x + y) + z == x + (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.wrapping_add(y) ==
                eval(u128 :: down((x.up() + y.up()) % (u128 :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() <= u128 :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(u128 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() > u128 :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(u128 :: down(x.up() + y.up() - u128 :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : u128, y : u128], precondition : true,
            postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : u128], precondition : true,
            postcondition : { let zero : u128 = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : u128, y : u128, z : u128],
            precondition : x.up() + y.up() + z.up() <= u128 :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() <= u128 :: MAX.up(),
            postcondition : x.checked_add(y) == Some(eval(u128 :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : u128, y : u128], precondition
                : x.up() + y.up() > u128 :: MAX.up(), postcondition : x.checked_add(y) ==
                None,
        },
        contract! {
            header : "Commutativity", inputs : [x : u128, y : u128], precondition : true,
            postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : u128], precondition : true,
            postcondition : x.checked_add(0u128) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : u128], precondition : true,
            postcondition : 0u128.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : u128, y : u128, z : u128],
            precondition : true, postcondition :
            x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs :
            [x : u128, y : u128], precondition : true, postcondition : x.saturating_add(y)
                == eval(u128 :: down((x.up() + y.up()).min(u128 :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() <= u128 :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(u128 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : u128, y : u128], precondition : x.up() + y.up() > u128 :: MAX.up(),
            postcondition : x.saturating_add(y) == u128 :: MAX,
        },
        contract! {
            header : "Semantics of rem", inputs : [x : usize, y : usize], precondition : y
                != 0, postcondition : x % y == eval(usize :: down(x.up() % y.up())),
        },
        contract! {
            header : "Semantics of rem", inputs : [x : usize], precondition : true,
            postcondition : { #[allow(unconditional_panic)] { panics! (x % 0) } }, n : 1,
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : usize, y : usize],
            precondition : y != 0, postcondition : x.checked_rem(y) ==
                Some(eval(usize :: down(x.up() % y.up()))),
        },
        contract! {
            header : "Semantics of checked_rem", inputs : [x : usize], precondition :
            true, postcondition :
            { #[allow(unconditional_panic)] { x.checked_rem(0) == None } }, n : 1,
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : usize],
            precondition : x == usize :: MIN, postcondition : x.checked_neg() ==
                Some(eval(usize :: down(- x.up()))), n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : usize], precondition : x !=
                usize :: MIN, postcondition : x.checked_neg() == None,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x << y == eval(usize :: down((x.up() << y) & usize :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            panics! (x << y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x.checked_shl(y) ==
                Some(eval(usize :: down((x.up() << y) & usize :: MAX.up()))), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            x.checked_shl(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x.overflowing_shl(y) ==
                (eval(usize :: down((x.up() << y) & usize :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the left shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            x.overflowing_shl(y) ==
                (eval(usize ::
                      down((x.up() << (y & (usize :: BITS - 1)) & usize :: MAX.up()))), true),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x >> y == eval(usize :: down((x.up() >> y) & usize :: MAX.up())), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            panics! (x >> y), strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x.checked_shr(y) ==
                Some(eval(usize :: down((x.up() >> y) & usize :: MAX.up()))), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            x.checked_shr(y) == None, strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift when the number of bits is right",
            inputs : [x : usize, y : u32], precondition : y < usize :: BITS, postcondition
                : x.overflowing_shr(y) ==
                (eval(usize :: down((x.up() >> y) & usize :: MAX.up())), false), strategy :
            Id_MicroInt,
        },
        contract! {
            header : "Semantics of the right shift otherwise", inputs :
            [x : usize, y : u32], precondition : y >= usize :: BITS, postcondition :
            x.overflowing_shr(y) ==
                (eval(usize ::
                      down((x.up() >> (y & (usize :: BITS - 1)) & usize :: MAX.up()))), true),
            strategy : Id_MicroInt,
        },
        contract! {
            header : "Semantics of the division by non-zero", inputs :
            [x : usize, y : usize], precondition : y != 0, postcondition : x / y ==
                eval(usize :: down(x.up() / y.up())), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the division by zero", inputs : [x : usize],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x / 0) } },
        },
        contract! {
            header : "Semantics of the saturating division by non-zero", inputs :
            [x : usize, y : usize], precondition : y != 0, postcondition :
            x.saturating_div(y) == eval(usize :: down(x.up() / y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the saturating division by zero", inputs : [x : usize],
            precondition : true, postcondition :
            { #[allow(unconditional_panic)] { panics! (x.saturating_div(0)) } },
        },
        contract! {
            header : "Semantics of the checked division by non-zero", inputs :
            [x : usize, y : usize], precondition : y != 0, postcondition :
            x.checked_div(y) == Some(eval(usize :: down(x.up() / y.up()))), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the checked division by zero", inputs : [x : usize],
            precondition : true, postcondition : x.checked_div(0) == None,
        },
        contract! {
            header : "Semantics of non-overflowing multiplication", inputs :
            [x : usize, y : usize], precondition : x.up() * y.up() <= usize :: MAX.up(),
            postcondition : x * y == eval(usize :: down(x.up() * y.up())), strategy :
            SmallInt_SmallInt,
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : usize, y : usize],
            precondition : x.up() * y.up() > usize :: MAX.up(), postcondition : panics!
                (x * y),
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : usize], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : usize, y : usize], precondition :
            x.up() * y.up() <= usize :: MAX.up(), postcondition : x * y == y * x, strategy
                : SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : usize, y : usize, z : usize],
            precondition :
            (x.up() * y.up() * z.up() <= usize :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : usize, y : usize, z : usize],
            precondition : (x.up() * (y.up() + z.up()) <= usize :: MAX.up() && x > 0),
            postcondition : x * (y + z) == x * y + x * z, strategy :
            SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : usize], precondition : true, postcondition : x
                * 0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of the non-overflowing checked multiplication", inputs :
            [x : usize, y : usize], precondition : x.up() * y.up() <= usize :: MAX.up(),
            postcondition : x.checked_mul(y) ==
                Some(eval(usize :: down(x.up() * y.up()))), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing checked multiplication", inputs :
            [x : usize, y : usize], precondition : x.up() * y.up() > usize :: MAX.up(),
            postcondition : x.checked_mul(y) == None,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when in bounds", inputs
                : [x : usize, y : usize], precondition : x.up() * y.up() <= usize :: MAX.up(),
            postcondition : x.overflowing_mul(y) ==
                (eval(usize :: down(x.up() * y.up())), false), strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing multiplication when out of bounds",
            inputs : [x : usize, y : usize], precondition : x.up() * y.up() > usize ::
            MAX.up(), postcondition : x.overflowing_mul(y) ==
                (eval(x.wrapping_mul(y)), true),
        },
        contract! {
            header : "Semantics of the saturating multiplication", inputs :
            [x : usize, y : usize], precondition : true, postcondition :
            x.saturating_mul(y) ==
                eval(usize :: down((x.up() * y.up()).min(usize :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating multiplication", inputs
                : [x : usize, y : usize], precondition : x.up() * y.up() <= usize :: MAX.up(),
            postcondition : x.saturating_mul(y) == eval(usize :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Semantics of the overflowing saturating multiplication", inputs :
            [x : usize, y : usize], precondition : x.up() * y.up() > usize :: MAX.up(),
            postcondition : x.saturating_mul(y) == usize :: MAX,
        },
        contract! {
            header : "Semantics of the wrapping multiplication", inputs :
            [x : usize, y : usize], precondition : true, postcondition : x.wrapping_mul(y)
                == eval(usize :: down((x.up() * y.up()) % (usize :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of the non-overflowing wrapping multiplication", inputs :
            [x : usize, y : usize], precondition : x.up() * y.up() <= usize :: MAX.up(),
            postcondition : x.wrapping_mul(y) == eval(usize :: down(x.up() * y.up())),
            strategy : SmallInt_SmallInt,
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x * 1 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : usize], precondition : true,
            postcondition : 1 * x == x,
        },
        contract! {
            header : "Commutativity", inputs : [x : usize, y : usize], precondition :
            x.up() * y.up() <= usize :: MAX.up(), postcondition : x * y == y * x, strategy
                : SmallInt_SmallInt,
        },
        contract! {
            header : "Associativity", inputs : [x : usize, y : usize, z : usize],
            precondition :
            (x.up() * y.up() * z.up() <= usize :: MAX.up() && x > 0 && y > 0 && z > 0),
            postcondition : (x * y) * z == x * (y * z), strategy :
            TinyInt_TinyInt_TinyInt,
        },
        contract! {
            header : "Distributivity", inputs : [x : usize, y : usize, z : usize],
            precondition : (x.up() * (y.up() + z.up()) <= usize :: MAX.up() && x > 0),
            postcondition : x * (y + z) == x * y + x * z, strategy :
            SmallInt_SmallInt_SmallInt,
        },
        contract! {
            header : "Zero", inputs : [x : usize], precondition : true, postcondition : x
                * 0 == 0, strategy : SmallInt,
        },
        contract! {
            header : "Semantics of non-underflowing subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x - y == eval(usize :: down(x.up() - y.up())),
        },
        contract! {
            header : "Panics when underflowing", inputs : [x : usize, y : usize],
            precondition : x.up() - y.up() < 0u8.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Subtraction is the reverse of addition", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : (x - y) + y == x,
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x - 0 == x,
        },
        contract! {
            header : "Semantics of non-underflowing wrapping subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.wrapping_sub(y) == eval(usize :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of underflowing wrapping subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() < 0u8.up(),
            postcondition : x.wrapping_sub(y) ==
                eval(usize :: down(x.up() - y.up() + usize :: MAX + 1)),
        },
        contract! {
            header : "Semantics of wrapping subtraction", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.wrapping_sub(y) ==
                eval(usize :: down((x.up() - y.up()).rem_euclid(& (usize :: MAX.up() + 1)))),
        },
        contract! {
            header : "Wrapping subtraction is the reverse of wrapping subtraction", inputs
                : [x : usize, y : usize], precondition : true, postcondition :
            (x.wrapping_sub(y)).wrapping_add(y) == x,
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x.wrapping_sub(0) == x,
        },
        contract! {
            header : "Semantics of non-underflowing checked subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.checked_sub(y) ==
                Some(eval(usize :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Semantics of underflowing checked subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() < 0u8.up(),
            postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Checked subtraction is the reverse of checked addition", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : (x.checked_sub(y)).and_then(| r | r.checked_add(y)) ==
                Some(x),
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x.checked_sub(0) == Some(x),
        },
        contract! {
            header : "Semantics of the saturating subtraction", inputs :
            [x : usize, y : usize], precondition : true, postcondition :
            x.saturating_sub(y) == eval(usize :: down((x.up() - y.up()).max(0u8.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() >= 0u8.up(),
            postcondition : x.saturating_sub(y) == eval(usize :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating subtraction", inputs :
            [x : usize, y : usize], precondition : x.up() - y.up() < 0u8.up(),
            postcondition : x.saturating_sub(y) == 0,
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() <= usize :: MAX.up(),
            postcondition : x + y == eval(usize :: down(x.up() + y.up())),
        },
        contract! {
            header : "Panics when overflowing", inputs : [x : usize, y : usize],
            precondition : x.up() + y.up() > usize :: MAX.up(), postcondition : panics!
                (x + y),
        },
        contract! {
            header : "Commutativity", inputs : [x : usize, y : usize], precondition :
            x.up() + y.up() <= usize :: MAX.up(), postcondition : x + y == y + x,
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x + 0 == x,
        },
        contract! {
            header : "Right identity", inputs : [x : usize], precondition : true,
            postcondition : 0 + x == x,
        },
        contract! {
            header : "Associativity", inputs : [x : usize, y : usize, z : usize],
            precondition : x.up() + y.up() + z.up() <= usize :: MAX.up(), postcondition :
            (x + y) + z == x + (y + z),
        },
        contract! {
            header : "Semantics of the wrapping addition", inputs :
            [x : usize, y : usize], precondition : true, postcondition : x.wrapping_add(y)
                == eval(usize :: down((x.up() + y.up()) % (usize :: MAX.up() + 1))),
        },
        contract! {
            header : "Semantics of non-overflowing wrapping addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() <= usize :: MAX.up(),
            postcondition : x.wrapping_add(y) == eval(usize :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing wrapping addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() > usize :: MAX.up(),
            postcondition : x.wrapping_add(y) ==
                eval(usize :: down(x.up() + y.up() - usize :: MAX - 1)),
        },
        contract! {
            header : "Commutativity", inputs : [x : usize, y : usize], precondition :
            true, postcondition : x.wrapping_add(y) == y.wrapping_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x.wrapping_add(0) == x,
        },
        contract! {
            header : "Right identity", inputs : [x : usize], precondition : true,
            postcondition : { let zero : usize = 0 ; zero.wrapping_add(x) == x },
        },
        contract! {
            header : "Associativity", inputs : [x : usize, y : usize, z : usize],
            precondition : x.up() + y.up() + z.up() <= usize :: MAX.up(), postcondition :
            (x.wrapping_add(y)).wrapping_add(z) == x.wrapping_add(y.wrapping_add(z)),
        },
        contract! {
            header : "Semantics of non-overflowing checked addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() <= usize :: MAX.up(),
            postcondition : x.checked_add(y) ==
                Some(eval(usize :: down(x.up() + y.up()))),
        },
        contract! {
            header : "None when overflowing", inputs : [x : usize, y : usize],
            precondition : x.up() + y.up() > usize :: MAX.up(), postcondition :
            x.checked_add(y) == None,
        },
        contract! {
            header : "Commutativity", inputs : [x : usize, y : usize], precondition :
            true, postcondition : x.checked_add(y) == y.checked_add(x),
        },
        contract! {
            header : "Left identity", inputs : [x : usize], precondition : true,
            postcondition : x.checked_add(0usize) == Some(x),
        },
        contract! {
            header : "Right identity", inputs : [x : usize], precondition : true,
            postcondition : 0usize.checked_add(x) == Some(x),
        },
        contract! {
            header : "Associativity", inputs : [x : usize, y : usize, z : usize],
            precondition : true, postcondition :
            x.checked_add(y).and_then(| iv | iv.checked_add(z)) ==
                y.checked_add(z).and_then(| iv | x.checked_add(iv)),
        },
        contract! {
            header : "Semantics of the saturating addition", inputs :
            [x : usize, y : usize], precondition : true, postcondition :
            x.saturating_add(y) ==
                eval(usize :: down((x.up() + y.up()).min(usize :: MAX.up()))),
        },
        contract! {
            header : "Semantics of the non-overflowing saturating addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() <= usize :: MAX.up(),
            postcondition : x.saturating_add(y) == eval(usize :: down(x.up() + y.up())),
        },
        contract! {
            header : "Semantics of the overflowing saturating addition", inputs :
            [x : usize, y : usize], precondition : x.up() + y.up() > usize :: MAX.up(),
            postcondition : x.saturating_add(y) == usize :: MAX,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i8], precondition : x != i8
                :: MIN, postcondition : x.checked_neg() == Some(eval(i8 :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i8],
            precondition : x == i8 :: MIN, postcondition : x.checked_neg() == None, n : 1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i8], precondition : x != i8
                :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(i8 :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i8],
            precondition : x == i8 :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : i8], precondition : x
                != i8 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i8 :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs : [x : i8],
            precondition : x == i8 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i8 :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() <= i8 :: MAX.up() && x.up() -
                y.up() >= i8 :: MIN.up(), postcondition : x - y ==
                eval(i8 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i8, y : i8],
            precondition : x.up() - y.up() > i8 :: MAX.up() || x.up() - y.up() < i8 ::
            MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() <= i8 :: MAX.up() && x.up() -
                y.up() >= i8 :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(i8 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i8, y : i8],
            precondition : x.up() - y.up() > i8 :: MAX.up() || x.up() - y.up() < i8 ::
            MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() <= i8 :: MAX.up() && x.up() -
                y.up() >= i8 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i8 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() > i8 :: MAX.up() || x.up() -
                y.up() < i8 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i8 ::
                     down((- i8 :: MIN.up() + x.up() - y.up()).rem_euclid(& (- i8 :: MIN.up() * 2))
                          + i8 :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() <= i8 :: MAX.up() && x.up() -
                y.up() >= i8 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i8 :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : i8, y : i8], precondition : x.up() - y.up() > i8 :: MAX.up() || x.up() -
                y.up() < i8 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i8 ::
                      down((- i8 :: MIN.up() + x.up() - y.up()).rem_euclid(& (- i8 :: MIN.up() * 2))
                           + i8 :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : i8, y : i8],
            precondition : x.up() + y.up() <= i8 :: MAX.up() && x.up() + y.up() >= i8 ::
            MIN.up(), postcondition : x + y == eval(i8 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : i8, y : i8],
            precondition : x.up() + y.up() > i8 :: MAX.up() || x.up() + y.up() < i8 ::
            MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i16], precondition : x !=
                i16 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(i16 :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i16],
            precondition : x == i16 :: MIN, postcondition : x.checked_neg() == None, n :
            1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i16], precondition : x !=
                i16 :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(i16 :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i16],
            precondition : x == i16 :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : i16], precondition : x
                != i16 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i16 :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs :
            [x : i16], precondition : x == i16 :: MIN, postcondition : x.overflowing_neg()
                == (eval(i16 :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() <= i16 :: MAX.up() &&
                x.up() - y.up() >= i16 :: MIN.up(), postcondition : x - y ==
                eval(i16 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i16, y : i16],
            precondition : x.up() - y.up() > i16 :: MAX.up() || x.up() - y.up() < i16 ::
            MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() <= i16 :: MAX.up() &&
                x.up() - y.up() >= i16 :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(i16 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i16, y : i16],
            precondition : x.up() - y.up() > i16 :: MAX.up() || x.up() - y.up() < i16 ::
            MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() <= i16 :: MAX.up() &&
                x.up() - y.up() >= i16 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i16 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() > i16 :: MAX.up() || x.up()
                - y.up() < i16 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i16 ::
                     down((- i16 :: MIN.up() + x.up() -
                           y.up()).rem_euclid(& (- i16 :: MIN.up() * 2)) + i16 :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() <= i16 :: MAX.up() &&
                x.up() - y.up() >= i16 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i16 :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : i16, y : i16], precondition : x.up() - y.up() > i16 :: MAX.up() || x.up()
                - y.up() < i16 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i16 ::
                      down((- i16 :: MIN.up() + x.up() -
                            y.up()).rem_euclid(& (- i16 :: MIN.up() * 2)) + i16 :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : i16, y : i16],
            precondition : x.up() + y.up() <= i16 :: MAX.up() && x.up() + y.up() >= i16 ::
            MIN.up(), postcondition : x + y == eval(i16 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : i16, y : i16],
            precondition : x.up() + y.up() > i16 :: MAX.up() || x.up() + y.up() < i16 ::
            MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i32], precondition : x !=
                i32 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(i32 :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i32],
            precondition : x == i32 :: MIN, postcondition : x.checked_neg() == None, n :
            1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i32], precondition : x !=
                i32 :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(i32 :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i32],
            precondition : x == i32 :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : i32], precondition : x
                != i32 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i32 :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs :
            [x : i32], precondition : x == i32 :: MIN, postcondition : x.overflowing_neg()
                == (eval(i32 :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() <= i32 :: MAX.up() &&
                x.up() - y.up() >= i32 :: MIN.up(), postcondition : x - y ==
                eval(i32 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i32, y : i32],
            precondition : x.up() - y.up() > i32 :: MAX.up() || x.up() - y.up() < i32 ::
            MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() <= i32 :: MAX.up() &&
                x.up() - y.up() >= i32 :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(i32 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i32, y : i32],
            precondition : x.up() - y.up() > i32 :: MAX.up() || x.up() - y.up() < i32 ::
            MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() <= i32 :: MAX.up() &&
                x.up() - y.up() >= i32 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i32 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() > i32 :: MAX.up() || x.up()
                - y.up() < i32 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i32 ::
                     down((- i32 :: MIN.up() + x.up() -
                           y.up()).rem_euclid(& (- i32 :: MIN.up() * 2)) + i32 :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() <= i32 :: MAX.up() &&
                x.up() - y.up() >= i32 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i32 :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : i32, y : i32], precondition : x.up() - y.up() > i32 :: MAX.up() || x.up()
                - y.up() < i32 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i32 ::
                      down((- i32 :: MIN.up() + x.up() -
                            y.up()).rem_euclid(& (- i32 :: MIN.up() * 2)) + i32 :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : i32, y : i32],
            precondition : x.up() + y.up() <= i32 :: MAX.up() && x.up() + y.up() >= i32 ::
            MIN.up(), postcondition : x + y == eval(i32 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : i32, y : i32],
            precondition : x.up() + y.up() > i32 :: MAX.up() || x.up() + y.up() < i32 ::
            MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i64], precondition : x !=
                i64 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(i64 :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i64],
            precondition : x == i64 :: MIN, postcondition : x.checked_neg() == None, n :
            1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i64], precondition : x !=
                i64 :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(i64 :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i64],
            precondition : x == i64 :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : i64], precondition : x
                != i64 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i64 :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs :
            [x : i64], precondition : x == i64 :: MIN, postcondition : x.overflowing_neg()
                == (eval(i64 :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() <= i64 :: MAX.up() &&
                x.up() - y.up() >= i64 :: MIN.up(), postcondition : x - y ==
                eval(i64 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i64, y : i64],
            precondition : x.up() - y.up() > i64 :: MAX.up() || x.up() - y.up() < i64 ::
            MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() <= i64 :: MAX.up() &&
                x.up() - y.up() >= i64 :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(i64 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i64, y : i64],
            precondition : x.up() - y.up() > i64 :: MAX.up() || x.up() - y.up() < i64 ::
            MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() <= i64 :: MAX.up() &&
                x.up() - y.up() >= i64 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i64 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() > i64 :: MAX.up() || x.up()
                - y.up() < i64 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i64 ::
                     down((- i64 :: MIN.up() + x.up() -
                           y.up()).rem_euclid(& (- i64 :: MIN.up() * 2)) + i64 :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() <= i64 :: MAX.up() &&
                x.up() - y.up() >= i64 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i64 :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : i64, y : i64], precondition : x.up() - y.up() > i64 :: MAX.up() || x.up()
                - y.up() < i64 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i64 ::
                      down((- i64 :: MIN.up() + x.up() -
                            y.up()).rem_euclid(& (- i64 :: MIN.up() * 2)) + i64 :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs : [x : i64, y : i64],
            precondition : x.up() + y.up() <= i64 :: MAX.up() && x.up() + y.up() >= i64 ::
            MIN.up(), postcondition : x + y == eval(i64 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : i64, y : i64],
            precondition : x.up() + y.up() > i64 :: MAX.up() || x.up() + y.up() < i64 ::
            MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i128], precondition : x !=
                i128 :: MIN, postcondition : x.checked_neg() ==
                Some(eval(i128 :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i128],
            precondition : x == i128 :: MIN, postcondition : x.checked_neg() == None, n :
            1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : i128], precondition : x !=
                i128 :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(i128 :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : i128],
            precondition : x == i128 :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : i128], precondition : x
                != i128 :: MIN, postcondition : x.overflowing_neg() ==
                (eval(i128 :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs :
            [x : i128], precondition : x == i128 :: MIN, postcondition :
            x.overflowing_neg() == (eval(i128 :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() <= i128 :: MAX.up() &&
                x.up() - y.up() >= i128 :: MIN.up(), postcondition : x - y ==
                eval(i128 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i128, y : i128],
            precondition : x.up() - y.up() > i128 :: MAX.up() || x.up() - y.up() < i128 ::
            MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() <= i128 :: MAX.up() &&
                x.up() - y.up() >= i128 :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(i128 :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : i128, y : i128],
            precondition : x.up() - y.up() > i128 :: MAX.up() || x.up() - y.up() < i128 ::
            MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() <= i128 :: MAX.up() &&
                x.up() - y.up() >= i128 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i128 :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() > i128 :: MAX.up() ||
                x.up() - y.up() < i128 :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(i128 ::
                     down((- i128 :: MIN.up() + x.up() -
                           y.up()).rem_euclid(& (- i128 :: MIN.up() * 2)) + i128 :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() <= i128 :: MAX.up() &&
                x.up() - y.up() >= i128 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i128 :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : i128, y : i128], precondition : x.up() - y.up() > i128 :: MAX.up() ||
                x.up() - y.up() < i128 :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(i128 ::
                      down((- i128 :: MIN.up() + x.up() -
                            y.up()).rem_euclid(& (- i128 :: MIN.up() * 2)) + i128 :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs :
            [x : i128, y : i128], precondition : x.up() + y.up() <= i128 :: MAX.up() &&
                x.up() + y.up() >= i128 :: MIN.up(), postcondition : x + y ==
                eval(i128 :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : i128, y : i128],
            precondition : x.up() + y.up() > i128 :: MAX.up() || x.up() + y.up() < i128 ::
            MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : isize], precondition : x !=
                isize :: MIN, postcondition : x.checked_neg() ==
                Some(eval(isize :: down(- x.up()))),
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : isize],
            precondition : x == isize :: MIN, postcondition : x.checked_neg() == None, n :
            1,
        },
        contract! {
            header : "Semantics of checked neg", inputs : [x : isize], precondition : x !=
                isize :: MIN, postcondition :
            { use std :: ops :: Neg ; x.neg() == eval(isize :: down(- x.up())) },
        },
        contract! {
            header : "Semantics of checked neg when out of bounds", inputs : [x : isize],
            precondition : x == isize :: MIN, postcondition :
            { use std :: ops :: Neg ; panics! (x.neg()) }, n : 1,
        },
        contract! {
            header : "Semantics of overflowing neg", inputs : [x : isize], precondition :
            x != isize :: MIN, postcondition : x.overflowing_neg() ==
                (eval(isize :: down(- x.up())), false),
        },
        contract! {
            header : "Semantics of overflowing neg when out of bounds", inputs :
            [x : isize], precondition : x == isize :: MIN, postcondition :
            x.overflowing_neg() == (eval(isize :: MIN), true), n : 1,
        },
        contract! {
            header : "Semantics of non-overflowing subtraction", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() <= isize :: MAX.up() &&
                x.up() - y.up() >= isize :: MIN.up(), postcondition : x - y ==
                eval(isize :: down(x.up() - y.up())),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : isize, y : isize],
            precondition : x.up() - y.up() > isize :: MAX.up() || x.up() - y.up() < isize
                :: MIN.up(), postcondition : panics! (x - y),
        },
        contract! {
            header : "Semantics of non-overflowing checked subtraction", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() <= isize :: MAX.up() &&
                x.up() - y.up() >= isize :: MIN.up(), postcondition : x.checked_sub(y) ==
                Some(eval(isize :: down(x.up() - y.up()))),
        },
        contract! {
            header : "Overflowing subtraction panics", inputs : [x : isize, y : isize],
            precondition : x.up() - y.up() > isize :: MAX.up() || x.up() - y.up() < isize
                :: MIN.up(), postcondition : x.checked_sub(y) == None,
        },
        contract! {
            header : "Semantics of non-overflowing wrapping subtraction", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() <= isize :: MAX.up() &&
                x.up() - y.up() >= isize :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(isize :: down(x.up() - y.up())),
        },
        contract! {
            header : "Semantics of overflowing wrapping subtraction", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() > isize :: MAX.up() ||
                x.up() - y.up() < isize :: MIN.up(), postcondition : x.wrapping_sub(y) ==
                eval(isize ::
                     down((- isize :: MIN.up() + x.up() -
                           y.up()).rem_euclid(& (- isize :: MIN.up() * 2)) + isize :: MIN.up())),
        },
        contract! {
            header : "Semantics of overflowing subtraction when in bounds", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() <= isize :: MAX.up() &&
                x.up() - y.up() >= isize :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(isize :: down(x.up() - y.up())), false),
        },
        contract! {
            header : "Semantics of overflowing subtraction when not in bounds", inputs :
            [x : isize, y : isize], precondition : x.up() - y.up() > isize :: MAX.up() ||
                x.up() - y.up() < isize :: MIN.up(), postcondition : x.overflowing_sub(y) ==
                (eval(isize ::
                      down((- isize :: MIN.up() + x.up() -
                            y.up()).rem_euclid(& (- isize :: MIN.up() * 2)) + isize :: MIN.up())), true),
        },
        contract! {
            header : "Semantics of non-overflowing addition", inputs :
            [x : isize, y : isize], precondition : x.up() + y.up() <= isize :: MAX.up() &&
                x.up() + y.up() >= isize :: MIN.up(), postcondition : x + y ==
                eval(isize :: down(x.up() + y.up())),
        },
        contract! {
            header : "Overflowing addition panics", inputs : [x : isize, y : isize],
            precondition : x.up() + y.up() > isize :: MAX.up() || x.up() + y.up() < isize
                :: MIN.up(), postcondition : panics! (x + y),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x ^ y == eval(u8 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x & y == eval(u8 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u8, y : u8], precondition :
            true, postcondition : x ^ y == eval(u8 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x ^ y == eval(u16 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x & y == eval(u16 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u16, y : u16], precondition
                : true, postcondition : x ^ y == eval(u16 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x ^ y == eval(u32 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x & y == eval(u32 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u32, y : u32], precondition
                : true, postcondition : x ^ y == eval(u32 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x ^ y == eval(u64 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x & y == eval(u64 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u64, y : u64], precondition
                : true, postcondition : x ^ y == eval(u64 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.cmp(& (y)) ==
                eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.lt(& (y)) ==
                eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.gt(& (y)) ==
                eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.ge(& (y)) ==
                eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x.le(& (y)) ==
                eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x ^ y ==
                eval(u128 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x & y ==
                eval(u128 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : u128, y : u128],
            precondition : true, postcondition : x ^ y ==
                eval(u128 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.cmp(& (y)) ==
                eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.lt(& (y)) ==
                eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.gt(& (y)) ==
                eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.ge(& (y)) ==
                eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x.le(& (y)) ==
                eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x ^ y ==
                eval(usize :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x & y ==
                eval(usize :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : usize, y : usize],
            precondition : true, postcondition : x ^ y ==
                eval(usize :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x ^ y == eval(i8 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x & y == eval(i8 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i8, y : i8], precondition :
            true, postcondition : x ^ y == eval(i8 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x ^ y == eval(i16 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x & y == eval(i16 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i16, y : i16], precondition
                : true, postcondition : x ^ y == eval(i16 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x ^ y == eval(i32 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x & y == eval(i32 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i32, y : i32], precondition
                : true, postcondition : x ^ y == eval(i32 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x.cmp(& (y)) == eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x.lt(& (y)) == eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x.gt(& (y)) == eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x.ge(& (y)) == eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x.le(& (y)) == eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x ^ y == eval(i64 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x & y == eval(i64 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i64, y : i64], precondition
                : true, postcondition : x ^ y == eval(i64 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x.cmp(& (y)) ==
                eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x.lt(& (y)) ==
                eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x.gt(& (y)) ==
                eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x.ge(& (y)) ==
                eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x.le(& (y)) ==
                eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x ^ y ==
                eval(i128 :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x & y ==
                eval(i128 :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : i128, y : i128],
            precondition : true, postcondition : x ^ y ==
                eval(i128 :: down(x.up() | y.up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x.cmp(& (y)) ==
                eval(x.up().cmp(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x.lt(& (y)) ==
                eval(x.up().lt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x.gt(& (y)) ==
                eval(x.up().gt(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x.ge(& (y)) ==
                eval(x.up().ge(& (y).up())),
        },
        contract! {
            header : "Semantics of comparaison", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x.le(& (y)) ==
                eval(x.up().le(& (y).up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x ^ y ==
                eval(isize :: down(x.up() ^ y.up())),
        },
        contract! {
            header : "Semantics of bitwise and", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x & y ==
                eval(isize :: down(x.up() & y.up())),
        },
        contract! {
            header : "Semantics of bitwise or", inputs : [x : isize, y : isize],
            precondition : true, postcondition : x ^ y ==
                eval(isize :: down(x.up() | y.up())),
        },
    ].into_iter().flatten().collect()
}
