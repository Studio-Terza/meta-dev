use syn::{GenericArgument, PathArguments, PathSegment, Type, TypePath};

#[derive(Clone, Copy)]
pub struct Written<'tree>(&'tree Type);

impl<'tree> Written<'tree> {
    const WRAPPERS: [&'static str; 2] = ["Option", "Result"];

    pub fn reading(tree: &'tree Type) -> Written<'tree> {
        Written(tree)
    }

    pub fn resource(self) -> Option<&'tree Type> {
        self.segment()
            .filter(|segment| segment.ident == "Held")
            .and_then(|segment| Self::first_type(&segment.arguments))
    }

    pub fn spine_end(self) -> Written<'tree> {
        self.wrapped().unwrap_or(self)
    }

    fn wrapped(self) -> Option<Written<'tree>> {
        self.segment()
            .filter(|segment| Self::WRAPPERS.iter().any(|name| segment.ident == *name))
            .and_then(|segment| Self::first_type(&segment.arguments))
            .map(Written)
    }

    fn segment(self) -> Option<&'tree PathSegment> {
        match self.0 {
            Type::Path(TypePath { qself: None, path }) => path.segments.last(),
            _ => None,
        }
    }

    fn first_type(arguments: &'tree PathArguments) -> Option<&'tree Type> {
        match arguments {
            PathArguments::AngleBracketed(angled) => angled.args.iter().find_map(Self::as_type),
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn as_type(argument: &'tree GenericArgument) -> Option<&'tree Type> {
        match argument {
            GenericArgument::Type(tree) => Some(tree),
            _ => None,
        }
    }
}
