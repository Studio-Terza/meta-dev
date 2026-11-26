use syntax::{
    Attribute, ExprMethodCall, Field, ForeignItemFn, ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl,
    ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, ItemUse, Local, Signature,
    TraitItemConst, TraitItemFn,
};

use lint_layout::Name;

pub trait Attributed {
    fn attributes(&self) -> &[Attribute];
}

pub trait Identified {
    fn identity(&self) -> Option<Name>;
}

macro_rules! attributed {
    ($($node:ty),*) => {
        $(impl Attributed for $node {
            fn attributes(&self) -> &[Attribute] {
                &self.attrs
            }
        })*
    };
}

attributed!(
    ItemMod,
    ItemImpl,
    ItemTrait,
    ItemFn,
    ImplItemFn,
    ItemEnum,
    ItemStruct,
    TraitItemFn,
    ItemType,
    ItemConst,
    ItemStatic,
    ItemUnion,
    ItemUse,
    ItemMacro,
    ImplItemConst,
    TraitItemConst,
    ForeignItemFn,
    Field,
    Local
);

macro_rules! identified {
    ($($node:ty => $($field:ident).+),*) => {
        $(impl Identified for $node {
            fn identity(&self) -> Option<Name> {
                Some(Name::from(&self$(.$field)+))
            }
        })*
    };
}

identified!(
    ItemMod => ident, ItemTrait => ident, ItemEnum => ident, ItemStruct => ident, ExprMethodCall => method,
    ItemType => ident, ItemUnion => ident, ItemConst => ident, ItemStatic => ident, Signature => ident,
    ItemFn => sig.ident, ImplItemFn => sig.ident, TraitItemFn => sig.ident, ForeignItemFn => sig.ident
);
