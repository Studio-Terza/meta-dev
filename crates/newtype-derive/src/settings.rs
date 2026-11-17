mod setting;

use syntax::Punctuated;
use syntax::parse::parse_quote;
use syntax::{Attribute, Expr, Stmt, Token, Type, WherePredicate};

pub use setting::Expressed;
use setting::Setting;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checking {
    Unchecked,
    Checked,
}

pub struct Settings {
    checking: Checking,
    declared: Vec<Setting>,
}

impl Settings {
    const ATTRIBUTE: &'static str = "newtype";

    pub fn declared(attributes: &[Attribute]) -> syntax::Result<Settings> {
        let lists = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| attribute.parse_args_with(Punctuated::<Setting, Token![,]>::parse_terminated))
            .collect::<syntax::Result<Vec<_>>>()?;
        let declared: Vec<Setting> = lists.into_iter().flatten().collect();
        let checking = declared.iter().find_map(Setting::checking).unwrap_or(Checking::Unchecked);
        Ok(Settings { checking, declared })
    }

    pub fn expressed(&self, wanted: Expressed) -> Option<&Expr> {
        self.declared.iter().find_map(|setting| setting.expressed(wanted))
    }

    pub fn required(&self, inner: &Type) -> Vec<WherePredicate> {
        match self.expressed(Expressed::Strategy) {
            None => vec![
                parse_quote!(Self: ::core::fmt::Debug),
                parse_quote!(#inner: ::newtype::proptest::arbitrary::Arbitrary + 'static),
                parse_quote!(<#inner as ::newtype::proptest::arbitrary::Arbitrary>::Strategy: 'static),
            ],
            Some(_) => vec![parse_quote!(Self: ::core::fmt::Debug), parse_quote!(Self: 'static)],
        }
    }

    pub fn parameters(&self, inner: &Type) -> Type {
        self.expressed(Expressed::Strategy).map_or_else(
            || parse_quote!(<#inner as ::newtype::proptest::arbitrary::Arbitrary>::Parameters),
            |_| parse_quote!(()),
        )
    }

    pub fn source(&self, inner: &Type) -> Vec<Stmt> {
        match self.expressed(Expressed::Strategy) {
            Some(strategy) => parse_quote!(let _ = parameters; let inner = #strategy;),
            None => parse_quote!(let inner = ::newtype::proptest::arbitrary::any_with::<#inner>(parameters);),
        }
    }

    pub fn checking(&self) -> Checking {
        self.checking
    }
}
