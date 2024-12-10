use super::*;

#[derive(fmt_derive::Debug, Clone, Hash, Eq, PartialEq)]
pub enum HaxQuery {
    Type {
        #[debug("{}", generics.into_token_stream())]
        generics: syn::Generics,
        #[debug("{}", generics.into_token_stream())]
        typ: syn::Type,
        #[debug(ignore)]
        use_statements: Vec<syn::ItemUse>,
    },
}

type Item = hax_frontend_exporter::Item<hax_frontend_exporter::ThirBody>;
use hax_frontend_exporter::{FnDef, ItemKind};

impl HaxQuery {
    fn ident_string(&self) -> String {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        format!("f{:016x}", s.finish())
    }
    fn ident(&self) -> syn::Ident {
        syn::Ident::new(&self.ident_string(), proc_macro2::Span::call_site())
    }
    fn result_from_item(&self, item: &Item) -> HaxQueryRes {
        match self {
            Self::Type { .. } => {
                let ItemKind::Fn(_, FnDef { params, .. }) = &item.kind else {
                    panic!("Invariant broken: expected function, got {item:?}");
                };
                let [p] = &params[..] else {
                    panic!("Invariant broken: function was expected to have exactly one parameter");
                };
                HaxQueryRes::Type(p.ty.clone())
            }
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum HaxQueryRes {
    Type(hax_frontend_exporter::Ty),
}

#[derive(Debug)]
pub struct HaxQueryWithId(HaxQuery, usize);

impl quote::ToTokens for HaxQueryWithId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = syn::Ident::new(&format!("item_{}", self.1), proc_macro2::Span::call_site());
        tokens.extend(match &self.0 {
            HaxQuery::Type {
                generics,
                typ,
                use_statements,
            } => {
                let where_clause = &generics.where_clause;
                quote! {
                    const _: () = {
                        #(#use_statements)*
                        fn #ident #generics(_: #typ) #where_clause {}
                    };
                }
            }
        })
    }
}

#[derive(Error, Debug)]
pub enum HaxQueryError {
    #[error(
        "hax failed to run on queries `{queries:#?}`, it returned the following errors:\n\n```\n{stderr}\n```"
    )]
    HaxError {
        queries: Vec<HaxQuery>,
        stderr: String,
    },
}

pub fn execute_hax_queries(
    queries: &[HaxQuery],
    deps: &HashMap<String, DependencySpec>,
) -> Result<Vec<HaxQueryRes>, HaxQueryError> {
    assert!(!queries.is_empty());
    let mut krate = Krate::new();
    let queries: Vec<_> = queries
        .into_iter()
        .enumerate()
        .map(|(i, query)| HaxQueryWithId(query.clone(), i))
        .collect();
    krate.add_dependencies(deps);
    let items: Vec<Item> = run_or_locate_error(&queries, |queries| {
        let source = quote! {
            #(#queries)*

            fn main(){}
        };
        krate.source(&format!("{source}"));
        krate.hax()
    })
    .map_err(|(queries, stderr)| HaxQueryError::HaxError {
        stderr,
        queries: queries.iter().map(|x| x.0.clone()).collect(),
    })?;
    use hax_frontend_exporter::{DefPathItem, DisambiguatedDefPathItem};
    let items: HashMap<_, Item> =
        HashMap::from_iter(items.into_iter().flat_map(|x| match &x.owner_id.path[..] {
            [.., DisambiguatedDefPathItem {
                data: DefPathItem::ValueNs(name),
                ..
            }] => Some((name.clone(), x.clone())),
            _ => None,
        }));
    let result: Vec<_> = queries
        .iter()
        .map(|HaxQueryWithId(query, i)| {
            query.result_from_item(
                items
                    .get(&format!("item_{}", i))
                    .expect("Internal error: an item was not emmited"),
            )
        })
        .collect();
    assert!(result.len() == queries.len());
    Ok(result)
}
