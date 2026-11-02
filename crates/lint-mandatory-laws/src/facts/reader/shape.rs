use syn::{Ident, ImplItem, ItemEnum, ItemImpl, ItemStruct, Type};

mod derives;
mod pointer_eq;

use super::super::exposure::Exposure;
use super::super::profile::Profile;
use crate::subject::Subject;
use derives::Derives;
use lint_core::Site;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_layout::Whole;
use lint_layout::{LineNumber, Word, words};
use pointer_eq::PointerEq;

#[derive(Debug)]
pub(crate) struct Shape;

impl Shape {
    const TRAITS: [Word<'static>; 6] = words!["Display", "FromStr", "Serialize", "Deserialize", "Ord", "PartialEq"];
    const METHODS: [Word<'static>; 2] = words!["encode", "decode"];
    const EQUALITY: [Word<'static>; 1] = words!["PartialEq"];
    const LEADS: [Word<'static>; 2] = words!["to", "from"];

    pub(crate) fn of_struct(file: &SourceFile, item: &ItemStruct) -> Profile {
        Self::declared(file, &item.ident, Exposure::declared_by(&item.vis), Derives::named(&item.attrs))
    }

    pub(crate) fn of_enum(file: &SourceFile, item: &ItemEnum) -> Profile {
        Self::declared(file, &item.ident, Exposure::declared_by(&item.vis), Derives::named(&item.attrs))
    }

    pub(crate) fn of_impl(file: &SourceFile, item: &ItemImpl) -> Option<Profile> {
        Self::owner(&item.self_ty).map(|owner| Self::declared(file, owner, Exposure::Private, Self::implemented(item)))
    }

    fn declared(file: &SourceFile, owner: &Ident, exposure: Exposure, facts: Vec<Name>) -> Profile {
        let site = Site::new(file.path(), LineNumber::from(owner.span()));
        Profile::declared(Subject::new(file, Name::from(owner)), site, exposure, facts)
    }

    fn owner(ty: &Type) -> Option<&Ident> {
        match ty {
            Type::Path(path) => path.path.segments.last().map(|last| &last.ident),
            _ => None,
        }
    }

    fn implemented(item: &ItemImpl) -> Vec<Name> {
        let implemented =
            item.trait_.iter().filter_map(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let by_address = PointerEq::is_whole_impl(item);
        implemented
            .filter(|name| name.matches::<Whole>(&Self::TRAITS))
            .filter(|name| !(by_address && name.matches::<Whole>(&Self::EQUALITY)))
            .chain(Self::conversions(&item.items))
            .collect()
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
