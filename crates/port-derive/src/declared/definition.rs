use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemTrait, Token, TraitItem, TraitItemFn, TypeParamBound, Visibility, parse_quote};

use super::method::Method;
use super::names::{CallerName, FillerName, OpsName};

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
    pub fn sealed_trait(&self) -> &ItemTrait {
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

    pub fn parsed(declared: ItemTrait) -> Declared {
        let mut item = Self::sealed(declared);
        let methods = item.items.iter_mut().filter_map(Self::required).map(Method::declared).collect();
        Declared {
            caller: CallerName::from(&item.ident),
            filler: FillerName::from(&item.ident),
            ops: OpsName::from(&item.ident),
            methods,
            item,
        }
    }

    fn sealed(mut item: ItemTrait) -> ItemTrait {
        let seal: TypeParamBound = parse_quote!(::port_core::Metered);
        item.colon_token.get_or_insert_with(<Token![:]>::default);
        item.supertraits.push(seal);
        item
    }

    fn required(item: &mut TraitItem) -> Option<&mut TraitItemFn> {
        match item {
            TraitItem::Fn(function) => Some(function).filter(|required| required.default.is_none()),
            _ => None,
        }
    }
}
