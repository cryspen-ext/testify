use syn::punctuated::Punctuated;
use syn::*;

pub struct NAryCall<const N: usize> {
    pub func: Box<Expr>,
    pub args: [Expr; N],
}

impl<const N: usize> TryFrom<&ExprCall> for NAryCall<N> {
    type Error = ();
    fn try_from(expr_call: &ExprCall) -> std::result::Result<Self, Self::Error> {
        let args: Vec<_> = expr_call.args.iter().cloned().collect();
        Ok(NAryCall {
            func: expr_call.func.clone(),
            args: args.try_into().map_err(|_| ())?,
        })
    }
}

impl<'a, const N: usize> TryFrom<&'a Expr> for NAryCall<N> {
    type Error = <Self as TryFrom<&'a ExprCall>>::Error;
    fn try_from(expr: &Expr) -> std::result::Result<Self, Self::Error> {
        match expr {
            Expr::Call(expr_call) => Self::try_from(expr_call),
            _ => Err(()),
        }
    }
}

#[extension_traits::extension(pub trait WhereClauseExt)]
impl WhereClause {
    fn merge_many(it: impl Iterator<Item = Self>) -> Self {
        let empty_where_clause = Self {
            where_token: Token![where](proc_macro2::Span::call_site()),
            predicates: punctuated::Punctuated::new(),
        };
        it.fold(empty_where_clause, |x, y| x.merge_one(&y))
    }
    fn merge_one(&self, other: &Self) -> Self {
        Self {
            where_token: self.where_token,
            predicates: self
                .predicates
                .iter()
                .cloned()
                .chain(other.predicates.iter().cloned())
                .collect(),
        }
    }
}

#[extension_traits::extension(pub trait PunctuatedExt)]
impl<T: Clone, Sep> Punctuated<T, Sep> {
    fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}

pub trait ExpectIdent {
    fn expect_ident(&self) -> Option<Ident>;
    fn is_ident(&self, name: &str) -> bool {
        self.expect_ident()
            .filter(|ident| &ident.to_string() == name)
            .is_some()
    }
}

impl<T: ExpectIdent> ExpectIdent for Box<T> {
    fn expect_ident(&self) -> Option<Ident> {
        let this: &T = &*self;
        this.expect_ident()
    }
}

fn expect_punctuated_1<T: Clone, S>(x: &Punctuated<T, S>) -> Option<T> {
    (x.len() == 1).then(|| x.first().unwrap().clone())
}

impl ExpectIdent for Path {
    fn expect_ident(&self) -> Option<Ident> {
        expect_punctuated_1(&self.segments).map(|s| s.ident)
    }
}

impl ExpectIdent for Expr {
    fn expect_ident(&self) -> Option<Ident> {
        match self {
            syn::Expr::Path(syn::ExprPath {
                qself: None, path, ..
            }) => path.expect_ident(),
            _ => None,
        }
    }
}

impl ExpectIdent for Type {
    fn expect_ident(&self) -> Option<Ident> {
        match self {
            syn::Type::Path(syn::TypePath {
                qself: None, path, ..
            }) => path.expect_ident(),
            _ => None,
        }
    }
}

impl ExpectIdent for Pat {
    fn expect_ident(&self) -> Option<Ident> {
        match self {
            syn::Pat::Ident(syn::PatIdent {
                by_ref: None,
                mutability: None,
                ident,
                subpat: None,
                ..
            }) => Some(ident.clone()),
            _ => None,
        }
    }
}
