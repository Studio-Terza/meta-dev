use syn::{Path, PathSegment};

use lint_layout::{Name, Whole, Word, words};

use super::TypeScope;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::resolved::Resolved;
use crate::spelling::Spelling;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Naming {
    Wrapper,
    Primitive,
    Itself,
    Declared,
}

impl Naming {
    const WRAPPERS: [Word<'static>; 14] = words![
        "Option", "Result", "Vec", "Box", "Rc", "Arc", "Cow", "VecDeque", "BTreeSet", "HashSet", "RefCell", "Cell",
        "Mutex", "Reverse"
    ];
    const PRIMITIVES: [Word<'static>; 18] = words![
        "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
        "char", "str", "String"
    ];
    const SELF: [Word<'static>; 1] = words!["Self"];

    pub(super) fn spelled(scope: &TypeScope<'_>, path: &Path, segment: &PathSegment) -> Resolved {
        let name = Name::from(&segment.ident);
        Naming::from(&name).resolved(scope, path, segment, name)
    }

    pub(super) fn resolved(self, scope: &TypeScope<'_>, path: &Path, segment: &PathSegment, name: Name) -> Resolved {
        match self {
            Naming::Wrapper => {
                scope.argument(&segment.arguments).unwrap_or_else(|| Resolved::Found(Endpoint::from(Unnamed::Unit)))
            }
            Naming::Primitive => Resolved::Found(Endpoint::new(Nature::Primitive, name)),
            Naming::Itself => Resolved::Itself,
            Naming::Declared => scope.declared(&name).unwrap_or_else(|| {
                Resolved::Found(Endpoint::new(Nature::Named, name).spelled(Spelling::of_path(path)))
            }),
        }
    }
}

impl From<&Name> for Naming {
    fn from(name: &Name) -> Naming {
        [
            (&Self::WRAPPERS[..], Naming::Wrapper),
            (&Self::PRIMITIVES[..], Naming::Primitive),
            (&Self::SELF[..], Naming::Itself),
        ]
        .into_iter()
        .find(|(spelled, _)| name.matches::<Whole>(spelled).is_some())
        .map_or(Naming::Declared, |(_, naming)| naming)
    }
}
