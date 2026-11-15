use syntax::{
    Attribute, ExprMethodCall, Field, ForeignItemFn, ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl,
    ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, ItemUse, LitFloat, LitInt, Local,
    Signature, TraitItemConst, TraitItemFn,
};

use lint_layout::{Name, Word, words};

pub trait Attributed {
    fn attributes(&self) -> &[Attribute];
}

pub trait Identified {
    fn identity(&self) -> Option<Name>;
}

pub trait Numbered {
    const UNITS: [Word<'static>; 6] = words!["0", "1", "0.0", "1.0", "-1", "-1.0"];

    fn digits(&self) -> &str;

    fn spot(&self) -> syntax::Span;
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

impl Identified for ItemImpl {
    fn identity(&self) -> Option<Name> {
        Name::of_type(&self.self_ty)
    }
}

impl Numbered for LitInt {
    fn digits(&self) -> &str {
        self.base10_digits()
    }

    fn spot(&self) -> syntax::Span {
        self.span()
    }
}

impl Numbered for LitFloat {
    fn digits(&self) -> &str {
        self.base10_digits()
    }

    fn spot(&self) -> syntax::Span {
        self.span()
    }
}
