mod boundary;
mod caller;
mod delegation;
mod filler;
mod method;
mod names;
mod ops;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Nothing;
use syn::{Attribute, Error, ItemTrait, TraitItem, Visibility};

use caller::Caller;
use delegation::Delegation;
use filler::Filler;
use method::Method;
use names::{CallerName, FillerName, OpsName};
use ops::Ops;

pub trait Rendered {
    fn rendered(declared: &Declared) -> TokenStream;

    fn rendered_after(declared: &Declared, earlier: &TokenStream) -> TokenStream {
        let own = Self::rendered(declared);
        quote! {
            #earlier
            #own
        }
    }
}

pub struct Declared {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    caller: CallerName,
    filler: FillerName,
    ops: OpsName,
    methods: Vec<Method>,
}

impl AsRef<[Method]> for Declared {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

impl Declared {
    const REFUSED: &'static str = "a #[port] trait has no generics, no supertraits and is neither unsafe nor auto";

    pub fn expanded(attributes: TokenStream, item: TokenStream) -> TokenStream {
        syn::parse2::<Nothing>(attributes)
            .and_then(|Nothing| syn::parse2::<ItemTrait>(item))
            .and_then(Declared::parsed)
            .map_or_else(|error| error.to_compile_error(), |declared| Self::expansion(&declared))
    }

    pub fn expansion(&self) -> TokenStream {
        let caller = Caller::rendered(self);
        let filler = Filler::rendered_after(self, &caller);
        let ops = Ops::rendered_after(self, &filler);
        Delegation::rendered_after(self, &ops)
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    pub fn caller_name(&self) -> &CallerName {
        &self.caller
    }

    pub fn filler_name(&self) -> &FillerName {
        &self.filler
    }

    pub fn ops_name(&self) -> &OpsName {
        &self.ops
    }

    fn parsed(item: ItemTrait) -> syn::Result<Declared> {
        Self::refuse_extras(&item)?;
        let ItemTrait { attrs, vis, ident, items, .. } = item;
        let methods = Self::methods_of(items)?;
        Ok(Declared {
            attributes: attrs,
            visibility: vis,
            caller: CallerName::from(&ident),
            filler: FillerName::from(&ident),
            ops: OpsName::from(&ident),
            methods,
        })
    }

    fn refuse_extras(item: &ItemTrait) -> syn::Result<()> {
        [
            item.generics.params.is_empty(),
            item.generics.where_clause.is_none(),
            item.supertraits.is_empty(),
            item.unsafety.is_none(),
            item.auto_token.is_none(),
        ]
        .into_iter()
        .all(std::convert::identity)
        .then_some(())
        .ok_or_else(|| Error::new_spanned(&item.ident, Self::REFUSED))
    }

    fn methods_of(items: Vec<TraitItem>) -> syn::Result<Vec<Method>> {
        items.into_iter().map(Self::method_of).collect()
    }

    fn method_of(item: TraitItem) -> syn::Result<Method> {
        match item {
            TraitItem::Fn(function) => Method::declared(function),
            other => Err(Error::new_spanned(other, "a #[port] trait holds methods only, no associated items")),
        }
    }
}
