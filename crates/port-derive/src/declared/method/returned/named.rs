use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, PathSegment, Type, TypePath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spine {
    Wrapper,
    Held,
}

#[derive(Clone, Copy)]
pub struct Named<'tree>(&'tree PathSegment);

pub struct NamedMut<'tree>(&'tree mut PathSegment);

pub struct Slot<'tree> {
    slot: &'tree mut Type,
    resource: Type,
}

impl Spine {
    const NAMES: [(&'static str, Spine); 3] =
        [("Option", Spine::Wrapper), ("Result", Spine::Wrapper), ("Held", Spine::Held)];

    pub fn named(segment: &PathSegment) -> Option<Spine> {
        Self::NAMES.iter().find(|(name, _)| segment.ident == name).map(|(_, spine)| *spine)
    }
}

impl<'tree> Named<'tree> {
    pub fn read(tree: &'tree Type) -> Option<Named<'tree>> {
        match tree {
            Type::Path(TypePath { qself: None, path }) => path.segments.last().map(Named),
            _ => None,
        }
    }

    pub fn resource(self) -> Option<&'tree Type> {
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
    pub fn read(tree: &'tree mut Type) -> Option<NamedMut<'tree>> {
        match tree {
            Type::Path(TypePath { qself: None, path }) => path.segments.last_mut().map(NamedMut),
            _ => None,
        }
    }

    pub fn held_slot(self) -> Option<Slot<'tree>> {
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

    pub fn unwrapped(mut rebuilt: Type) -> Option<Type> {
        NamedMut::read(&mut rebuilt).and_then(NamedMut::held_slot).map(Slot::filled)?;
        Some(rebuilt)
    }

    fn filled(self) {
        *self.slot = self.resource;
    }
}
