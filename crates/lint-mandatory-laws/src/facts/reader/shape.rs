use syn::{Ident, ImplItem, ItemEnum, ItemImpl, ItemStruct, Type};

mod derives;

use super::super::exposure::Exposure;
use super::super::profile::Profile;
use crate::subject::Subject;
use derives::Derives;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_layout::Name;
use lint_layout::Whole;
use lint_layout::{LineNumber, Word, words};

#[derive(Debug)]
pub(crate) struct Shape;

impl Shape {
    const TRAITS: [Word<'static>; 4] = words!["Display", "FromStr", "Serialize", "Deserialize"];
    const METHODS: [Word<'static>; 2] = words!["encode", "decode"];
    const LEADS: [Word<'static>; 2] = words!["to", "from"];

    pub(crate) fn of_struct<'file>(file: &'file SourceFile, item: &ItemStruct) -> Profile<'file> {
        Self::declared(file, &item.ident, Exposure::declared_by(&item.vis), Derives::named(&item.attrs))
    }

    pub(crate) fn of_enum<'file>(file: &'file SourceFile, item: &ItemEnum) -> Profile<'file> {
        Self::declared(file, &item.ident, Exposure::declared_by(&item.vis), Derives::named(&item.attrs))
    }

    pub(crate) fn of_impl<'file>(file: &'file SourceFile, item: &ItemImpl) -> Option<Profile<'file>> {
        Self::owner(&item.self_ty).map(|owner| Self::declared(file, owner, Exposure::Private, Self::implemented(item)))
    }

    fn declared<'file>(file: &'file SourceFile, owner: &Ident, exposure: Exposure, facts: Vec<Name>) -> Profile<'file> {
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
        implemented
            .filter(|name| name.matches::<Whole>(&Self::TRAITS).is_some())
            .chain(item.items.iter().filter_map(Self::conversion))
            .collect()
    }

    fn conversion(member: &ImplItem) -> Option<Name> {
        let method = Syntax::impl_fn(member).map(|function| Name::from(&function.sig.ident))?;
        method
            .matches::<Whole>(&Self::METHODS)
            .or_else(|| method.words().first().and_then(|first| first.matches::<Whole>(&Self::LEADS)))
            .map(|_| method)
    }
}
