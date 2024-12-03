use num_bigint::*;
pub use num_traits::ops::euclid::Euclid;

pub trait Lift: Sized {
    type Abstract;
    fn up(self) -> Self::Abstract;
    fn down(a: Self::Abstract) -> Self;
}

impl<T> Lift for Vec<T> {
    type Abstract = Vec<T>;
    fn up(self) -> Self::Abstract {
        self
    }
    fn down(x: Self::Abstract) -> Self {
        x
    }
}

impl Lift for () {
    type Abstract = ();
    fn up(self) -> Self::Abstract {
        ()
    }
    fn down(_: Self::Abstract) -> Self {
        ()
    }
}

use std::sync::Arc;

#[derive(Clone)]
pub struct Fn1<I, O> {
    pub f: Arc<dyn Fn(I) -> O>,
    pub string: String,
}

#[derive(Clone)]
pub struct FnR1<I, O> {
    pub f: Arc<dyn for<'a> Fn(&'a I) -> O>,
    pub string: String,
}

impl PrintRust for core::cmp::Ordering {
    fn print_as_rust(&self) -> String {
        format!("core::cmp::Ordering::{:#?}", self)
    }
    fn print_type() -> String {
        format!("core::cmp::Ordering")
    }
}

impl<I: PrintRust, O: PrintRust> PrintRust for FnR1<I, O> {
    fn print_as_rust(&self) -> String {
        format!("({})", self.string)
    }
    fn print_type() -> String {
        format!("Fn(&{}) -> {}", I::print_type(), O::print_type())
    }
}

impl<I: PrintRust, O: PrintRust> PrintRust for Fn1<I, O> {
    fn print_as_rust(&self) -> String {
        format!("({})", self.string)
    }
    fn print_type() -> String {
        format!("Fn({}) -> {}", I::print_type(), O::print_type())
    }
}

impl<I, O> std::hash::Hash for Fn1<I, O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        false.hash(state);
        self.string.hash(state);
    }
}

impl<I, O> std::hash::Hash for FnR1<I, O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        true.hash(state);
        self.string.hash(state);
    }
}

impl<I, O> Eq for Fn1<I, O> {}
impl<I, O> PartialEq for Fn1<I, O> {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl<I, O> Eq for FnR1<I, O> {}
impl<I, O> PartialEq for FnR1<I, O> {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

macro_rules! mkfn {
    (|$pat:ident : $ty:ty| $body:expr) => {
        Fn1::<$ty, _> {
            f: Arc::new(move |$pat: $ty| $body),
            string: format!("{}", stringify!(|$pat: $ty| $body)),
        }
    };
    ($str:expr, |$pat:ident : $ty:ty| $body:expr) => {
        Fn1::<$ty, _> {
            f: Arc::new(move |$pat: $ty| $body),
            string: $str,
        }
    };
}

macro_rules! mkfnr {
    ($str:expr, |$pat:ident : &$ty:ty| $body:expr) => {
        FnR1::<$ty, _> {
            f: Arc::new(move |$pat: &$ty| $body),
            string: $str,
        }
    };
}

trait IntType: num_traits::int::PrimInt {}
impl IntType for u8 {}
impl IntType for i8 {}
impl IntType for u16 {}
impl IntType for i16 {}
impl IntType for u32 {}
impl IntType for i32 {}
impl IntType for u64 {}
impl IntType for i64 {}
impl IntType for u128 {}
impl IntType for i128 {}
impl IntType for usize {}
impl IntType for isize {}

/// When this trait is implemented for a type `T`, it allows to print
/// any value of type `T` as a valid Rust expression, but also to
/// print the type `T` as a valid Rust type.
pub trait PrintRust {
    /// Prints a value of type `Self` as a valid Rust expression.
    fn print_as_rust(&self) -> String;
    /// Prints the type `Self` as a valid Rust type.
    fn print_type() -> String;
}

impl PrintRust for bool {
    fn print_as_rust(&self) -> String {
        format!("{}", self)
    }
    fn print_type() -> String {
        "bool".into()
    }
}

impl<T: PrintRust> PrintRust for Vec<T> {
    fn print_as_rust(&self) -> String {
        if self.is_empty() {
            format!("Vec::<{}>::new()", T::print_type())
        } else {
            format!(
                "vec![{}]",
                self.iter()
                    .map(T::print_as_rust)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
    fn print_type() -> String {
        format!("Vec<{}>", T::print_type())
    }
}

impl<T: PrintRust> PrintRust for Option<T> {
    fn print_as_rust(&self) -> String {
        if self.is_none() {
            format!("Option::<{}>::None", T::print_type())
        } else {
            match self {
                Some(value) => format!("Some({})", value.print_as_rust()),
                None => format!("None"),
            }
        }
    }
    fn print_type() -> String {
        format!("Option<{}>", T::print_type())
    }
}

impl<A: PrintRust, B: PrintRust> PrintRust for (A, B) {
    fn print_as_rust(&self) -> String {
        format!("({}, {})", self.0.print_as_rust(), self.1.print_as_rust())
    }
    fn print_type() -> String {
        format!("({}, {})", A::print_type(), B::print_type())
    }
}
impl<A: PrintRust, B: PrintRust, C: PrintRust> PrintRust for (A, B, C) {
    fn print_as_rust(&self) -> String {
        format!(
            "({}, {}, {})",
            self.0.print_as_rust(),
            self.1.print_as_rust(),
            self.2.print_as_rust()
        )
    }
    fn print_type() -> String {
        format!(
            "({}, {}, {})",
            A::print_type(),
            B::print_type(),
            C::print_type()
        )
    }
}

macro_rules! derive_for_newtype {
    ($ty:ident) => {
        impl<T: PrintRust> PrintRust for $ty<T> {
            fn print_as_rust(&self) -> String {
                T::print_as_rust(&self.0)
            }
            fn print_type() -> String {
                T::print_type()
            }
        }

        impl<T: Lift> Lift for $ty<T> {
            type Abstract = T::Abstract;
            fn up(self) -> Self::Abstract {
                self.0.up()
            }
            fn down(x: Self::Abstract) -> Self {
                Self(T::down(x))
            }
        }

        impl<T> UnwrapStrategyPoly<T> for $ty<T> {
            fn unwrap_strategy_poly(self) -> T {
                self.0
            }
        }
    };
}

pub trait UnwrapStrategyPoly<Unwrapped> {
    fn unwrap_strategy_poly(self) -> Unwrapped;
}

impl<UA, UB, A: UnwrapStrategyPoly<UA>, B: UnwrapStrategyPoly<UB>> UnwrapStrategyPoly<(UA, UB)>
    for (A, B)
{
    fn unwrap_strategy_poly(self) -> (UA, UB) {
        (self.0.unwrap_strategy_poly(), self.1.unwrap_strategy_poly())
    }
}
impl<
        UA,
        UB,
        UC,
        A: UnwrapStrategyPoly<UA>,
        B: UnwrapStrategyPoly<UB>,
        C: UnwrapStrategyPoly<UC>,
    > UnwrapStrategyPoly<(UA, UB, UC)> for (A, B, C)
{
    fn unwrap_strategy_poly(self) -> (UA, UB, UC) {
        (
            self.0.unwrap_strategy_poly(),
            self.1.unwrap_strategy_poly(),
            self.2.unwrap_strategy_poly(),
        )
    }
}

#[allow(dead_code)]
mod combinations {
    #![allow(non_camel_case_types)]
    use super::*;
    pub type SmallInt_SmallInt<T, U> = (SmallInt<T>, SmallInt<U>);
    pub type SmallInt_SmallInt_SmallInt<T, U, V> = (SmallInt<T>, SmallInt<U>, SmallInt<V>);
    pub type TinyInt_TinyInt_TinyInt<T, U, V> = (TinyInt<T>, TinyInt<U>, TinyInt<V>);
    pub type TinyInt_TinyInt<T, U> = (TinyInt<T>, TinyInt<U>);
    pub type Id_TinyInt<T, U> = (IdStrategy<T>, TinyInt<U>);
    pub type Id_MicroInt<T, U> = (IdStrategy<T>, MicroInt<U>);
}
pub use combinations::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SmallInt<T>(pub T);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TinyInt<T>(pub T);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MicroInt<T>(pub T);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdStrategy<T>(pub T);

derive_for_newtype!(SmallInt);
derive_for_newtype!(TinyInt);
derive_for_newtype!(MicroInt);
derive_for_newtype!(IdStrategy);

macro_rules! print_rust_num_lit {
    ($ty:ident) => {
        impl PrintRust for $ty {
            fn print_as_rust(&self) -> String {
                if self < &0 {
                    format!("({}{})", self, stringify!($ty))
                }else {
                    format!("{}{}", self, stringify!($ty))
                }
            }
            fn print_type() -> String {
                format!("{}", stringify!($ty))
            }
        }
        impl Lift for $ty {
            type Abstract = BigInt;
            fn up(self) -> Self::Abstract {
                BigInt::from(self)
            }
            fn down(x: Self::Abstract) -> Self {
                use num_traits::cast::ToPrimitive;
                paste::paste! { x.[<to_ $ty>]().expect(&format!("BigInt {x} is not fitting type {}", stringify!($ty))) }
            }
        }
    };
}

print_rust_num_lit!(u8);
print_rust_num_lit!(u16);
print_rust_num_lit!(u32);
print_rust_num_lit!(u64);
print_rust_num_lit!(u128);
print_rust_num_lit!(usize);
print_rust_num_lit!(i8);
print_rust_num_lit!(i16);
print_rust_num_lit!(i32);
print_rust_num_lit!(i64);
print_rust_num_lit!(i128);
print_rust_num_lit!(isize);

#[macro_export]
macro_rules! catch_unwind_set_hook {
    ($f:expr) => {{
        use std::panic;
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind($f);
        panic::set_hook(previous_hook);
        result
    }};
}

#[macro_export]
macro_rules! doesn_t_panic {
    ($e:expr) => {
        catch_unwind_set_hook!(|| $e).is_ok()
    };
}

#[macro_export]
macro_rules! panics {
    ($e:expr) => {
        catch_unwind_set_hook!(|| $e).is_err()
    };
}

#[macro_export]
macro_rules! catch_panic {
    ($e:expr) => {
        catch_unwind_set_hook!(|| $e).ok()
    };
}
