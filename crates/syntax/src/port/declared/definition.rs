use syn::{ItemTrait, Token, TraitItem, TraitItemFn, TypeParamBound, Visibility, parse_quote};

use super::method::Method;
use super::names::{CallerRole, FillerRole, Name, OpsRole};

#[derive(Debug)]
pub struct Declared {
    item: ItemTrait,
    caller: Name<CallerRole>,
    filler: Name<FillerRole>,
    ops: Name<OpsRole>,
    methods: Vec<Method>,
}

impl AsRef<[Method]> for Declared {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

impl Declared {
    pub(crate) fn sealed_trait(&self) -> &ItemTrait {
        &self.item
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        &self.item.vis
    }

    pub(crate) fn caller_name(&self) -> &Name<CallerRole> {
        &self.caller
    }

    pub(crate) fn filler_name(&self) -> &Name<FillerRole> {
        &self.filler
    }

    pub(crate) fn ops_name(&self) -> &Name<OpsRole> {
        &self.ops
    }

    pub fn parsed(declared: ItemTrait) -> Declared {
        let mut item = Self::sealed(declared);
        let methods = item.items.iter_mut().filter_map(Self::required).map(Method::declared).collect();
        Declared {
            caller: Name::from(&item.ident),
            filler: Name::from(&item.ident),
            ops: Name::from(&item.ident),
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
