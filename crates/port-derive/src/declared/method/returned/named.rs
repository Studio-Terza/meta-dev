use strum::{IntoStaticStr, VariantArray};
use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, PathSegment, Type, TypePath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Spine {
    Wrapper,
    Held,
}

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelled {
    Option,
    Result,
    Held,
}

#[derive(Clone, Copy)]
pub(crate) struct Named<'tree>(&'tree PathSegment);

pub(crate) struct NamedMut<'tree>(&'tree mut PathSegment);

pub(crate) struct Slot<'tree> {
    slot: &'tree mut Type,
    resource: Type,
}

impl Spine {
    pub(crate) fn named(segment: &PathSegment) -> Option<Spine> {
        Spelled::VARIANTS.iter().find(|spelled| segment.ident == <&str>::from(**spelled)).map(|spelled| spelled.spine())
    }
}

impl Spelled {
    fn spine(self) -> Spine {
        match self {
            Spelled::Option | Spelled::Result => Spine::Wrapper,
            Spelled::Held => Spine::Held,
        }
    }
}

impl<'tree> Named<'tree> {
    pub(crate) fn read(tree: &'tree Type) -> Option<Named<'tree>> {
        match tree {
            Type::Path(TypePath { qself: None, path }) => path.segments.last().map(Named),
            _ => None,
        }
    }

    pub(crate) fn resource(self) -> Option<&'tree Type> {
        Some(self.0)
            .filter(|segment| Spine::named(segment) == Some(Spine::Held))
            .and_then(|segment| Self::angled(&segment.arguments))
            .and_then(|angled| angled.args.iter().filter_map(Self::typed).next_back())
    }

    fn angled(arguments: &PathArguments) -> Option<&AngleBracketedGenericArguments> {
        match arguments {
            PathArguments::AngleBracketed(angled) => Some(angled),
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn typed(argument: &GenericArgument) -> Option<&Type> {
        match argument {
            GenericArgument::Type(tree) => Some(tree),
            _ => None,
        }
    }
}

impl<'tree> NamedMut<'tree> {
    pub(crate) fn read(tree: &'tree mut Type) -> Option<NamedMut<'tree>> {
        match tree {
            Type::Path(TypePath { qself: None, path }) => path.segments.last_mut().map(NamedMut),
            _ => None,
        }
    }

    pub(crate) fn held_slot(self) -> Option<Slot<'tree>> {
        Some(self.0)
            .filter(|segment| Spine::named(segment) == Some(Spine::Wrapper))
            .and_then(|segment| Self::angled(&mut segment.arguments))
            .and_then(|angled| angled.args.iter_mut().find_map(Self::typed))
            .and_then(Slot::held)
    }

    fn angled(arguments: &mut PathArguments) -> Option<&mut AngleBracketedGenericArguments> {
        match arguments {
            PathArguments::AngleBracketed(angled) => Some(angled),
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn typed(argument: &mut GenericArgument) -> Option<&mut Type> {
        match argument {
            GenericArgument::Type(tree) => Some(tree),
            _ => None,
        }
    }
}

impl<'tree> Slot<'tree> {
    fn held(slot: &'tree mut Type) -> Option<Slot<'tree>> {
        let found = Named::read(slot).and_then(Named::resource).cloned();
        found.map(|resource| Slot { slot, resource })
    }

    pub(crate) fn unwrapped(mut rebuilt: Type) -> Option<Type> {
        NamedMut::read(&mut rebuilt).and_then(NamedMut::held_slot).map(Slot::filled)?;
        Some(rebuilt)
    }

    fn filled(self) {
        *self.slot = self.resource;
    }
}
