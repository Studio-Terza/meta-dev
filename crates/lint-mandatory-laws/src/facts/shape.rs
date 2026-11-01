use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, ImplItem, ItemEnum, ItemImpl, ItemStruct, Path, Token, Type};

use super::profile::Profile;
use lint_layout::Name;
use lint_layout::Whole;

#[derive(Debug)]
pub(crate) struct Shape;

impl Shape {
    const DERIVE: &'static str = "derive";
    const TRAITS: [&'static str; 6] = ["Display", "FromStr", "Serialize", "Deserialize", "Ord", "PartialEq"];
    const DERIVED: [&'static str; 2] = ["Serialize", "Deserialize"];
    const METHODS: [&'static str; 2] = ["encode", "decode"];
    const LEADS: [&'static str; 2] = ["to", "from"];

    pub(crate) fn of_struct(item: &ItemStruct) -> Profile {
        Profile::declared(&item.ident, Self::derived(&item.attrs))
    }

    pub(crate) fn of_enum(item: &ItemEnum) -> Profile {
        Profile::declared(&item.ident, Self::derived(&item.attrs))
    }

    pub(crate) fn of_impl(item: &ItemImpl) -> Option<Profile> {
        Self::owner(&item.self_ty).map(|owner| Profile::declared(owner, Self::implemented(item)))
    }

    fn derived(attributes: &[Attribute]) -> Vec<Name> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .flatten()
            .filter_map(|path| path.segments.last().map(|last| Name::from(&last.ident)))
            .filter(|name| name.matches::<Whole>(&Self::DERIVED))
            .collect()
    }

    fn owner(ty: &Type) -> Option<&Ident> {
        match ty {
            Type::Path(path) => path.path.segments.last().map(|last| &last.ident),
            _ => None,
        }
    }

    fn implemented(item: &ItemImpl) -> Vec<Name> {
        match &item.trait_ {
            Some((_, path, _)) => path
                .segments
                .last()
                .map(|last| Name::from(&last.ident))
                .filter(|name| name.matches::<Whole>(&Self::TRAITS))
                .into_iter()
                .collect(),
            None => Self::conversions(&item.items),
        }
    }

    fn conversions(members: &[ImplItem]) -> Vec<Name> {
        members
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(function) => Some(Name::from(&function.sig.ident)),
                _ => None,
            })
            .filter(|method| {
                method.matches::<Whole>(&Self::METHODS)
                    || method.words().first().is_some_and(|first| first.matches::<Whole>(&Self::LEADS))
            })
            .collect()
    }
}
