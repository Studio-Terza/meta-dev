use syn::{ItemImpl, Type};

use lint_core::Syntax;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owner {
    name: Name,
    argument: Option<Name>,
}

impl Owner {
    pub(crate) fn declared(name: Name) -> Owner {
        Owner { name, argument: None }
    }

    pub(crate) fn of_impl(found: &ItemImpl) -> Option<Owner> {
        let argument = found
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|arguments| arguments.args.iter().find_map(Syntax::type_argument))
            .and_then(Self::named);
        Self::named(&found.self_ty).map(|name| Owner { name, argument })
    }

    #[must_use]
    pub fn argument(&self) -> Option<&Name> {
        self.argument.as_ref()
    }

    fn named(found: &Type) -> Option<Name> {
        match found {
            Type::Reference(reference) => Self::named(&reference.elem),
            _ => Name::of_type(found),
        }
    }
}

impl AsRef<Name> for Owner {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
