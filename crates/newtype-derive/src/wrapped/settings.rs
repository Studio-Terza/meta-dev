mod setting;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Expr, Token};

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
        match (function, error) {
            (Some(function), Some(error)) => Ok(Settings { check: Some(Check::new(function, error)), strategy }),
            (None, None) => Ok(Settings { check: None, strategy }),
            (Some(_), None) | (None, Some(_)) => Err(Error::new(Span::call_site(), Self::INCOMPLETE)),
        }
    }

    pub fn checking(&self) -> Option<&Check> {
        self.check.as_ref()
    }

    pub fn strategy(&self) -> Option<&Expr> {
        self.strategy.as_ref()
    }
}
