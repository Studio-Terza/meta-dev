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
use syn::{ItemTrait, TraitItem, TraitItemFn, Visibility};

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
    item: ItemTrait,
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
    pub fn expanded(attributes: TokenStream, item: TokenStream) -> TokenStream {
        syn::parse2::<Nothing>(attributes)
            .and_then(|Nothing| syn::parse2::<ItemTrait>(item))
            .map(Declared::parsed)
            .map_or_else(|error| error.to_compile_error(), |declared| Self::expansion(&declared))
    }

    pub fn expansion(&self) -> TokenStream {
        let caller = Caller::rendered(self);
        let filler = Filler::rendered_after(self, &caller);
        let ops = Ops::rendered_after(self, &filler);
        Delegation::rendered_after(self, &ops)
    }

    pub fn declared_trait(&self) -> &ItemTrait {
        &self.item
    }

    pub fn visibility(&self) -> &Visibility {
        &self.item.vis
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

    fn parsed(item: ItemTrait) -> Declared {
        let methods =
            item.items.iter().filter_map(Self::required).map(|function| Method::declared(&function.sig)).collect();
        Declared {
            caller: CallerName::from(&item.ident),
            filler: FillerName::from(&item.ident),
            ops: OpsName::from(&item.ident),
            methods,
            item,
        }
    }

    fn required(item: &TraitItem) -> Option<&TraitItemFn> {
        match item {
            TraitItem::Fn(function) => function.default.is_none().then_some(function),
            _ => None,
        }
    }
}
