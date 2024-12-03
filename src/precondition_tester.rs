use crate::prelude::*;
use itertools::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::process::{Child, Command};
use std::rc::Rc;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Precondition {
    pub inputs: Vec<(syn::Ident, syn::Type)>,
    pub predicate: syn::Expr,
}
