use crate::prelude::*;
use extension_traits::extension;

#[extension(pub trait DefIdExt)]
impl hax_frontend_exporter::DefId {
    fn into_string(&self) -> String {
        use core::ops::Deref;
        DefIdContentsExt::into_string(self.deref())
    }
}

#[extension(pub trait DefIdContentsExt)]
impl hax_frontend_exporter::DefIdContents {
    fn into_string(&self) -> String {
        std::iter::once(self.krate.to_string())
            .chain(self.path.iter().flat_map(|i| {
                use hax_frontend_exporter::DefPathItem;
                Some(
                    match &i.data {
                        DefPathItem::TypeNs(s) | DefPathItem::ValueNs(s) => s,
                        DefPathItem::Impl => "r#impl",
                        DefPathItem::Use => "r#use",
                        DefPathItem::AnonConst => "r#_",
                        DefPathItem::LifetimeNs(_) => "'lifetime",
                        DefPathItem::ForeignMod => "r#foreign_mod",
                        _ => return None,
                    }
                    .to_string(),
                )
            }))
            .join("::")
    }
}

#[extension(pub trait ItemExt)]
impl<B: hax_frontend_exporter::IsBody + Serialize> hax_frontend_exporter::Item<B> {
    /// Finds the `def_id`s mentionned in an item.
    fn def_ids(&self) -> Vec<hax_frontend_exporter::DefId> {
        use serde_json::Value;
        let mut def_ids: Vec<hax_frontend_exporter::DefId> = vec![];
        let mut queue = vec![serde_json::to_value(self).unwrap()];
        while let Some(json) = queue.pop() {
            if let Ok(def_id) = serde_json::from_value(json.clone()) {
                def_ids.push(def_id);
            };
            match json {
                Value::Null | Value::Number(_) | Value::String(_) | Value::Bool(_) => (),
                Value::Array(values) => queue.extend(values),
                Value::Object(map) => queue.extend(map.values().cloned()),
            }
        }
        def_ids
    }
}

#[extension(pub trait SpanExt)]
impl hax_frontend_exporter::Span {
    fn file_contents(&self, workdir: &Path) -> Option<String> {
        std::fs::read_to_string(workdir.join(self.filename.to_path()?)).ok()
    }
    fn source(&self, workdir: &Path) -> Option<String> {
        Some(
            self.file_contents(workdir)?
                .lines()
                .enumerate()
                .map(|(n, s)| (n + 1, s))
                .filter(|(n, _)| *n >= self.lo.line && *n <= self.hi.line)
                .map(|(_, s)| s)
                .join("\n"),
        )
    }
}

#[extension(pub trait StrExt)]
impl<'a> &'a str {
    /// `line` is not 0-based but 1-based: the first line of a file is denoted `1`
    fn split_at_line_col(self, line: usize, col: usize) -> (String, String) {
        let lines: Vec<_> = self.lines().collect();
        assert!(!lines.is_empty());
        assert!(line >= 1);
        let [first_lines @ .., middle_line] = &lines[..line] else {
            panic!()
        };
        let first_lines = first_lines.join("\n");
        let last_lines = if lines.len() > line {
            lines[line + 1..].join("\n")
        } else {
            "".to_string()
        };
        let middle_line_l: String = middle_line.chars().take(col).collect();
        let middle_line_r: String = middle_line.chars().skip(col).collect();
        (
            format!("{first_lines}{middle_line_l}"),
            format!("{middle_line_r}{last_lines}"),
        )
    }
    fn split_at_loc(self, loc: hax_frontend_exporter::Loc) -> (String, String) {
        self.split_at_line_col(loc.line, loc.col)
    }
}

pub mod serde_via {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt::Display;

    pub trait SerdeVia: Clone {
        type Repr: Serialize + for<'de> Deserialize<'de>;
        fn to_repr(self) -> Self::Repr;
        fn from_repr(repr: Self::Repr) -> Result<Self, impl Display>;
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            Self::to_repr(self.clone()).serialize(s)
        }
        fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            use serde::de::Error;
            Self::from_repr(Self::Repr::deserialize(d)?).map_err(|err| D::Error::custom(err))
        }
    }

    trait AutoSerdeVia: quote::ToTokens + syn::parse::Parse + Clone {}

    impl<T: AutoSerdeVia> SerdeVia for T {
        type Repr = String;
        fn from_repr(v: Self::Repr) -> Result<Self, impl Display> {
            use std::str::FromStr;
            let ts = proc_macro2::TokenStream::from_str(&v)?;
            syn::parse2(ts)
        }
        fn to_repr(self) -> Self::Repr {
            use quote::ToTokens;
            format!("{}", self.into_token_stream())
        }
    }

    impl AutoSerdeVia for syn::Path {}
    impl AutoSerdeVia for syn::Type {}
    impl AutoSerdeVia for syn::Expr {}
    impl AutoSerdeVia for syn::WhereClause {}
    impl AutoSerdeVia for syn::ItemUse {}

    impl SerdeVia for super::Span {
        type Repr = u8;
        fn from_repr(v: Self::Repr) -> Result<Self, impl Display> {
            Ok::<_, &str>(Self::dummy())
        }
        fn to_repr(self) -> Self::Repr {
            0
        }
    }
    impl<T: SerdeVia> SerdeVia for Option<T> {
        type Repr = Option<T::Repr>;
        fn from_repr(v: Self::Repr) -> Result<Self, impl Display> {
            match v {
                Some(v) => T::from_repr(v).map(Some),
                None => Ok(None),
            }
        }
        fn to_repr(self) -> Self::Repr {
            self.map(T::to_repr)
        }
    }
    impl<T: SerdeVia> SerdeVia for Vec<T> {
        type Repr = Vec<T::Repr>;
        fn from_repr(v: Self::Repr) -> Result<Self, impl Display> {
            v.into_iter().map(T::from_repr).collect()
        }
        fn to_repr(self) -> Self::Repr {
            self.into_iter().map(T::to_repr).collect()
        }
    }

    use std::collections::HashMap;
    impl<T: SerdeVia> SerdeVia for HashMap<String, T> {
        type Repr = HashMap<String, T::Repr>;
        fn from_repr(v: Self::Repr) -> Result<Self, impl Display> {
            v.into_iter()
                .map(|(k, v)| T::from_repr(v).map(|v| (k, v)))
                .collect::<Result<Vec<_>, _>>()
                .map(|v| Self::from_iter(v.into_iter()))
        }
        fn to_repr(self) -> Self::Repr {
            self.into_iter().map(|(k, v)| (k, T::to_repr(v))).collect()
        }
    }
}

pub fn dependencies_to_string(deps: &HashMap<String, DependencySpec>) -> String {
    let mut wrapper = HashMap::new();
    wrapper.insert("dependencies", deps);
    toml::to_string(&wrapper).unwrap()
}

pub fn default_expr() -> syn::Expr {
    parse_quote! {true}
}
