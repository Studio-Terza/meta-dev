mod setting;

use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Stmt, Token, Type, WherePredicate, parse_quote};

use setting::Setting;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checking {
    Unchecked,
    Checked,
}

pub struct Settings {
    checking: Checking,
    strategy: Option<Expr>,
}

impl Settings {
    const ATTRIBUTE: &'static str = "newtype";

    pub fn declared(attributes: &[Attribute]) -> syn::Result<Settings> {
        let lists = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| attribute.parse_args_with(Punctuated::<Setting, Token![,]>::parse_terminated))
            .collect::<syn::Result<Vec<_>>>()?;
        let settings: Vec<Setting> = lists.into_iter().flatten().collect();
        let checking = settings.iter().find_map(Setting::checking).unwrap_or(Checking::Unchecked);
        let strategy = settings.iter().find_map(Setting::strategy).cloned();
        Ok(Settings { checking, strategy })
    }

    pub fn required(&self, inner: &Type) -> Vec<WherePredicate> {
        match self.strategy {
            None => vec![
                parse_quote!(Self: ::core::fmt::Debug),
                parse_quote!(#inner: ::newtype::proptest::arbitrary::Arbitrary + 'static),
                parse_quote!(<#inner as ::newtype::proptest::arbitrary::Arbitrary>::Strategy: 'static),
            ],
            Some(_) => vec![parse_quote!(Self: ::core::fmt::Debug), parse_quote!(Self: 'static)],
        }
    }

    pub fn parameters(&self, inner: &Type) -> Type {
        self.strategy.as_ref().map_or_else(
            || parse_quote!(<#inner as ::newtype::proptest::arbitrary::Arbitrary>::Parameters),
            |_| parse_quote!(()),
        )
    }

    pub fn source(&self, inner: &Type) -> Vec<Stmt> {
        match &self.strategy {
            Some(strategy) => parse_quote!(let _ = parameters; let inner = #strategy;),
            None => parse_quote!(let inner = ::newtype::proptest::arbitrary::any_with::<#inner>(parameters);),
        }
    }

    pub fn checking(&self) -> Checking {
        self.checking
    }
}
