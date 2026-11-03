mod setting;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Expr, Stmt, Token, Type, WherePredicate, parse_quote};

use super::check::Check;
use setting::Setting;

pub struct Settings {
    check: Option<Check>,
    strategy: Option<Expr>,
}

impl Settings {
    const ATTRIBUTE: &'static str = "newtype";
    const INCOMPLETE: &'static str = "a check needs an error, and an error needs a check";

    pub fn declared(attributes: &[Attribute]) -> syn::Result<Settings> {
        let lists = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| attribute.parse_args_with(Punctuated::<Setting, Token![,]>::parse_terminated))
            .collect::<syn::Result<Vec<_>>>()?;
        let settings: Vec<Setting> = lists.into_iter().flatten().collect();
        let function = settings.iter().find_map(Setting::function).cloned();
        let error = settings.iter().find_map(Setting::error).cloned();
        let strategy = settings.iter().find_map(Setting::strategy).cloned();
        let partial = function.as_ref().map(|_| ()).xor(error.as_ref().map(|_| ()));
        partial.map_or_else(
            || {
                Ok(Settings {
                    check: function.zip(error).map(|(function, error)| Check::new(function, error)),
                    strategy,
                })
            },
            |()| Err(Error::new(Span::call_site(), Self::INCOMPLETE)),
        )
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

    pub fn checking(&self) -> Option<&Check> {
        self.check.as_ref()
    }
}
