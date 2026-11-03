mod pointer;

use std::collections::BTreeSet;

use syn::{Ident, PathSegment, Type};

use lint_layout::Name;
use pointer::Pointer;

#[derive(Debug)]
pub struct Spelling;

impl Spelling {
    pub fn inner(ty: &Type) -> Option<&Type> {
        match ty {
            Type::Reference(reference) => Some(reference.elem.as_ref()),
            _ => Self::segment(ty).and_then(Pointer::pointee),
        }
    }

    pub fn ident(ty: &Type) -> Option<&Ident> {
        match ty {
            Type::Path(path) => path.path.get_ident(),
            _ => None,
        }
    }

    pub fn generic(ty: &Type, generics: &BTreeSet<Name>) -> Option<Name> {
        Self::ident(ty).map(Name::from).filter(|name| generics.contains(name))
    }

    pub fn last(ty: &Type) -> Option<Name> {
        Self::segment(Self::peeled(ty)).map(|last| Name::from(&last.ident))
    }

    fn peeled(ty: &Type) -> &Type {
        match ty {
            Type::Reference(reference) => Self::peeled(&reference.elem),
            _ => ty,
        }
    }

    fn segment(ty: &Type) -> Option<&PathSegment> {
        match ty {
            Type::Path(path) => path.path.segments.last(),
            _ => None,
        }
    }
}
