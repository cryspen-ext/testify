use syn::punctuated::Punctuated;
use syn::*;

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
