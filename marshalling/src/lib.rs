pub type ValueRepr = serde_json::Value;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExpectError {
    #[error("Got `ValueRepr::List`, but got tag `{got}` instead of expected `{expected}`")]
    BadTag { got: String, expected: String },

    #[error("Expected `ValueRepr::List`, got {repr:?}")]
    BadVariant { repr: ValueRepr },

    #[error("Got `ValueRepr::List`, but got {got} values instead of expected {expected} values")]
    BadArity { expected: usize, got: usize },
}

pub trait ValueReprAPI: Sized {
    fn expect_tagged(&self, tag_: &str) -> Result<&[Self], ExpectError>;
    fn expect_tagged_n<const N: usize>(&self, tag: &str) -> Result<&[Self; N], ExpectError> {
        let vec = self.expect_tagged(tag)?;
        let len = vec.len();
        vec.try_into().map_err(|_| ExpectError::BadArity {
            expected: N,
            got: len,
        })
    }
    fn mk_tagged(tag: &str, data: &[Self]) -> Self;
}

impl ValueReprAPI for serde_json::Value {
    fn expect_tagged(&self, tag_: &str) -> Result<&[Self], ExpectError> {
        let serde_json::Value::Object(map) = self else {
            Err(ExpectError::BadVariant { repr: self.clone() })?
        };
        let Some(serde_json::Value::Array(data)) = map.get("data") else {
            Err(ExpectError::BadVariant { repr: self.clone() })?
        };
        let Some(serde_json::Value::String(tag)) = map.get("tag") else {
            Err(ExpectError::BadVariant { repr: self.clone() })?
        };
        if tag != tag_ {
            Err(ExpectError::BadTag {
                got: tag.to_string(),
                expected: tag_.to_string(),
            })?
        }
        Ok(data.as_slice())
    }
    fn mk_tagged(tag: &str, data: &[Self]) -> Self {
        Self::Object(serde_json::Map::from_iter(
            [
                ("tag".to_string(), Self::String(tag.to_string())),
                ("data".to_string(), Self::Array(data.to_vec())),
            ]
            .into_iter(),
        ))
    }
}

pub use bumpalo::Bump as Arena;

pub trait ToValueRepr {
    fn to_value_repr(&self) -> ValueRepr;
}
pub trait FromValueRepr<'a> {
    fn from_value_repr(repr: &ValueRepr, arena: &'a Arena) -> Self;
}
pub trait ToRustExpr {
    fn to_rust_expr(&self) -> String;
    fn to_rust_type() -> String;
}

mod to_rust_expr_primitive_types {
    use super::*;

    macro_rules! impl_to_rust_expr_via_format {
        ($($t:ident $format_str:literal),*$(,)?) => {
            $(
                impl ToRustExpr for $t {
                    fn to_rust_expr(&self) -> String {
                        format!($format_str, self)
                    }
                    fn to_rust_type() -> String {
                        stringify!($t).to_string()
                    }
                }
            )*
        }
    }

    impl<T: ToRustExpr> ToRustExpr for Vec<T> {
        fn to_rust_expr(&self) -> String {
            format!(
                "[{}]",
                self.iter()
                    .map(T::to_rust_expr)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        fn to_rust_type() -> String {
            format!("Vec<{}>", T::to_rust_type())
        }
    }

    impl<T: ToRustExpr, const N: usize> ToRustExpr for [T; N] {
        fn to_rust_expr(&self) -> String {
            if N == 0 {
                format!("([] as [0; {}])", T::to_rust_type())
            } else {
                format!(
                    "[{}]",
                    self.iter()
                        .map(T::to_rust_expr)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        fn to_rust_type() -> String {
            format!("[{}]", T::to_rust_type())
        }
    }

    impl<T: ToRustExpr> ToRustExpr for &[T] {
        fn to_rust_expr(&self) -> String {
            if self.is_empty() {
                format!("(&[] as &[{}])", T::to_rust_type())
            } else {
                format!(
                    "&[{}]",
                    self.iter()
                        .map(T::to_rust_expr)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        fn to_rust_type() -> String {
            format!("[{}]", T::to_rust_type())
        }
    }

    impl<T: ToRustExpr> ToRustExpr for Option<T> {
        fn to_rust_expr(&self) -> String {
            match self {
                None => format!("None::<{}>", T::to_rust_type()),
                Some(value) => format!("Some({})", value.to_rust_expr()),
            }
        }
        fn to_rust_type() -> String {
            format!("[{}]", T::to_rust_type())
        }
    }

    impl<T: ToRustExpr> ToRustExpr for &T {
        fn to_rust_expr(&self) -> String {
            (*self).to_rust_expr()
        }
        fn to_rust_type() -> String {
            format!("&{}", T::to_rust_type())
        }
    }

    impl_to_rust_expr_via_format!(
        usize "{}usize", u8 "{}u8", u16 "{}u16", u32 "{}u32", u64 "{}u64", u128 "{}u128",
        isize "{}isize", i8 "{}i8", i16 "{}i16", i32 "{}i32", i64 "{}i64", i128 "{}i128",
        String "{:?}", char "{:?}", f32 "{}", f64 "{}", bool "{:?}", str "{:?}",
    );
}

mod value_repr_primitive_types {
    use super::*;
    const SLICE_TAG: &str = "::slice";
    const TUPLE_TAG: &str = "::tuple";
    const OPTION_SOME_TAG: &str = "std::option::Option::Some";
    const OPTION_NONE_TAG: &str = "std::option::Option::None";

    impl<'a, T: FromValueRepr<'a>> FromValueRepr<'a> for Vec<T> {
        fn from_value_repr(repr: &ValueRepr, arena: &'a Arena) -> Self {
            let reprs = repr.expect_tagged(SLICE_TAG).unwrap();
            reprs
                .iter()
                .map(|item| T::from_value_repr(item, arena))
                .collect()
        }
    }
    impl<'a, T: ToValueRepr> ToValueRepr for &'a [T] {
        fn to_value_repr(&self) -> ValueRepr {
            ValueRepr::mk_tagged(
                SLICE_TAG,
                &self.iter().map(T::to_value_repr).collect::<Vec<_>>(),
            )
        }
    }
    impl<T: ToValueRepr> ToValueRepr for Vec<T> {
        fn to_value_repr(&self) -> ValueRepr {
            <&[T]>::to_value_repr(&&*self.as_ref())
        }
    }

    impl<'a> ToValueRepr for &'a str {
        fn to_value_repr(&self) -> ValueRepr {
            self.to_string().to_value_repr()
        }
    }

    impl<'a, T: FromValueRepr<'a>> FromValueRepr<'a> for &'a T {
        fn from_value_repr(repr: &ValueRepr, arena: &'a Arena) -> Self {
            let x = T::from_value_repr(repr, arena);
            arena.alloc(x)
        }
    }

    impl<T: ToValueRepr> ToValueRepr for Option<T> {
        fn to_value_repr(&self) -> ValueRepr {
            match self {
                Some(value) => ValueRepr::mk_tagged(OPTION_SOME_TAG, &[value.to_value_repr()]),
                None => ValueRepr::mk_tagged(OPTION_NONE_TAG, &[]),
            }
        }
    }
    impl<'a, T: FromValueRepr<'a>> FromValueRepr<'a> for Option<T> {
        fn from_value_repr(repr: &ValueRepr, arena: &'a Arena) -> Self {
            if let Ok([repr]) = repr.expect_tagged_n(OPTION_SOME_TAG) {
                Some(T::from_value_repr(repr, arena))
            } else {
                let [] = repr.expect_tagged_n(OPTION_NONE_TAG).unwrap();
                None
            }
        }
    }

    macro_rules! impl_from_value_repr_via_as_ref {
        ( $lt:lifetime $t:ty {$u:ty} $({{$($generics:tt)*} $($where:tt)*})? $(, $($tt:tt)*)? ) => {
            impl<$lt, $($($generics)*)?> FromValueRepr<'a> for &$lt $t $($($where)*)? {
                fn from_value_repr(repr: &ValueRepr, arena: &$lt Arena) -> Self {
                    let u: &$u = <&$u>::from_value_repr(repr, arena);
                    use ::std::convert::AsRef as _;
                    u.as_ref()
                }
            }

            $(impl_from_value_repr_via_as_ref!($lt $($tt)*);)?
        };
        ($lt:lifetime) => {}
    }

    impl_from_value_repr_via_as_ref!('a str { String }, [T] { Vec<T> } {{T: FromValueRepr<'a>}});

    macro_rules! impl_from_value_repr_via_serde {
        ( $t:ty $({{$($generics:tt)*} $($where:tt)*})? $(, $($tt:tt)*)? ) => {
            impl<'a, $($($generics)*)?> FromValueRepr<'a> for $t $($($where)*)? {
                fn from_value_repr(repr: &ValueRepr, _arena: &'a Arena) -> Self {
                    ::serde_json::from_value(repr.clone()).unwrap()
                }
            }
            impl<$($($generics)*)?> ToValueRepr for $t $($($where)*)? {
                fn to_value_repr(&self) -> ValueRepr {
                    ::serde_json::to_value(self).unwrap()
                }
            }
            $(impl_from_value_repr_via_serde!($($tt)*);)?
        };
        () => {}
    }

    impl_from_value_repr_via_serde!(u8, u16, u32, u64, usize);
    impl_from_value_repr_via_serde!(i8, i16, i32, i64, isize);
    impl_from_value_repr_via_serde!(String, bool, char, f32, f64);

    impl<'a> FromValueRepr<'a> for u128 {
        fn from_value_repr(repr: &ValueRepr, _arena: &'a Arena) -> Self {
            let s: String = ::serde_json::from_value(repr.clone()).unwrap();
            s.parse().unwrap()
        }
    }
    impl ToValueRepr for u128 {
        fn to_value_repr(&self) -> ValueRepr {
            serde_json::Value::String(format!("{self}"))
        }
    }
    impl<'a> FromValueRepr<'a> for i128 {
        fn from_value_repr(repr: &ValueRepr, _arena: &'a Arena) -> Self {
            let s: String = ::serde_json::from_value(repr.clone()).unwrap();
            s.parse().unwrap()
        }
    }
    impl ToValueRepr for i128 {
        fn to_value_repr(&self) -> ValueRepr {
            serde_json::Value::String(format!("{self}"))
        }
    }

    mod ordering {
        use super::*;
        const LT: &str = "Less";
        const EQ: &str = "Equal";
        const GT: &str = "Greater";
        use std::cmp::Ordering;
        impl ToValueRepr for Ordering {
            fn to_value_repr(&self) -> ValueRepr {
                match self {
                    Ordering::Less => ValueRepr::mk_tagged(LT, &[]),
                    Ordering::Equal => ValueRepr::mk_tagged(EQ, &[]),
                    Ordering::Greater => ValueRepr::mk_tagged(GT, &[]),
                }
            }
        }
        impl<'a> FromValueRepr<'a> for Ordering {
            fn from_value_repr(repr: &ValueRepr, _: &'a Arena) -> Self {
                use std::cmp::Ordering;
                if repr.expect_tagged_n::<0>(LT).is_ok() {
                    Ordering::Less
                } else if repr.expect_tagged_n::<0>(EQ).is_ok() {
                    Ordering::Equal
                } else {
                    repr.expect_tagged_n::<0>(GT).unwrap();
                    Ordering::Greater
                }
            }
        }

        impl ToRustExpr for Ordering {
            fn to_rust_expr(&self) -> String {
                format!(
                    "{}::{}",
                    Self::to_rust_type(),
                    match self {
                        Ordering::Less => "Less",
                        Ordering::Equal => "Equal",
                        Ordering::Greater => "Greater",
                    }
                )
            }
            fn to_rust_type() -> String {
                format!("std::cmp::Ordering")
            }
        }
    }

    macro_rules! impl_tuple_value_repr {
        (@) => {};
        (@$a:ident $av:ident $(, $t:ident $tv:ident)*) => {
            impl_tuple_value_repr!(@$($t $tv),*);
            impl_tuple_value_repr!($a $av $(,$t $tv)*);
        };
        ($($t:ident $v:ident),*) => {
            impl<'a, $($t: FromValueRepr<'a>,)*> FromValueRepr<'a> for ($($t,)*) {
                fn from_value_repr(repr: &ValueRepr, arena: &'a Arena) -> Self {
                    const TUPLE_SIZE: usize = { $(({ let _ = |$v:()| $v; 1 }) + )* 0 };
                    let [$($v),*] = repr.expect_tagged_n::<TUPLE_SIZE>(TUPLE_TAG).unwrap();
                    ($($t::from_value_repr($v, arena),)*)
                }
            }
            impl<$($t: ToValueRepr,)*> ToValueRepr for ($($t,)*) {
                fn to_value_repr(&self) -> ValueRepr {
                    let ($($v,)*) = self;
                    ValueRepr::mk_tagged(
                        TUPLE_TAG,
                        &[$($t::to_value_repr($v),)*],
                    )
                }
            }
            impl<$($t: ToRustExpr,)*> ToRustExpr for ($($t,)*) {
                fn to_rust_expr(&self) -> String {
                    let ($($v,)*) = self;
                    format!("({})", [$($v.to_rust_expr()),*].join(","))
                }
                fn to_rust_type() -> String {
                    format!("({})", [$($t::to_rust_type()),*].join(","))
                }
            }
        }
    }

    impl_tuple_value_repr!(@T0 v0, T1 v1, T2 v2, T3 v3, T4 v4, T5 v5, T6 v6, T7 v7, T8 v8, T9 v9, T10 v10, T11 v11, T12 v12, T13 v13, T14 v14, T15 v15, T16 v16, T17 v17, T18 v18, T19 v19, T20 v20);
}

#[test]
fn test() {
    let x: &[_] = &[Some(1u8), None];
    println!("{}", x.to_rust_expr())
}
