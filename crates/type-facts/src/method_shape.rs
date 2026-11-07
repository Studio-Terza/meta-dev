use std::fmt;

use syn::{Signature, Type, TypeParamBound};

use lint_layout::{Name, Whole, Word, words};

use crate::spelled::Spelled;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Receiver {
    Absent,
    Owned,
    Shared,
    Mutable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Taken {
    Itself,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Returned {
    Nothing,
    Itself,
    Optional,
    Fallible,
    Many,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodShape {
    receiver: Receiver,
    takes: Vec<Taken>,
    returned: Returned,
}

impl MethodShape {
    const SELF: [Word<'static>; 1] = words!["Self"];

    #[must_use]
    pub fn of_signature(signature: &Signature, selfish: Option<&Name>) -> Self {
        let receiver = signature.receiver().map_or(Receiver::Absent, Receiver::from);
        let takes = signature
            .inputs
            .iter()
            .filter_map(Spelled::typed)
            .map(|typed| {
                Spelled::last_of(&typed.ty).and_then(|name| Self::itself(&name, selfish)).unwrap_or(Taken::Other)
            })
            .collect();
        let returned = Spelled::returned_by(&signature.output)
            .map_or(Returned::Nothing, |found| Returned::of_type(found, selfish));
        Self { receiver, takes, returned }
    }

    #[must_use]
    pub const fn receiver(&self) -> Receiver {
        self.receiver
    }

    #[must_use]
    pub fn takes(&self) -> &[Taken] {
        &self.takes
    }

    #[must_use]
    pub const fn returned(&self) -> Returned {
        self.returned
    }

    fn itself(name: &Name, selfish: Option<&Name>) -> Option<Taken> {
        name.matches::<Whole>(&Self::SELF)
            .map(|_| Taken::Itself)
            .or_else(|| selfish.filter(|own| *own == name).map(|_| Taken::Itself))
    }
}

impl From<&syn::Receiver> for Receiver {
    fn from(found: &syn::Receiver) -> Self {
        found.reference.as_ref().map_or(Self::Owned, |_| found.mutability.map_or(Self::Shared, |_| Self::Mutable))
    }
}

impl Returned {
    const OPTIONAL: [Word<'static>; 1] = words!["Option"];
    const FALLIBLE: [Word<'static>; 1] = words!["Result"];
    const MANY: [Word<'static>; 10] = words![
        "Vec",
        "VecDeque",
        "BTreeSet",
        "HashSet",
        "BTreeMap",
        "HashMap",
        "BinaryHeap",
        "Iterator",
        "IntoIterator",
        "DoubleEndedIterator"
    ];

    fn of_type(found: &Type, selfish: Option<&Name>) -> Self {
        match found {
            Type::Reference(reference) => Self::of_type(&reference.elem, selfish),
            Type::Paren(paren) => Self::of_type(&paren.elem, selfish),
            Type::Tuple(tuple) => tuple.elems.first().map_or(Self::Nothing, |_| Self::Value),
            Type::Slice(_) | Type::Array(_) => Self::Many,
            Type::ImplTrait(bounds) => bounds.bounds.iter().find_map(Self::bounded).unwrap_or(Self::Value),
            Type::Path(_) => Spelled::last_of(found).map_or(Self::Value, |name| Self::named(&name, selfish)),
            _ => Self::Value,
        }
    }

    fn bounded(bound: &TypeParamBound) -> Option<Self> {
        match bound {
            TypeParamBound::Trait(found) => found
                .path
                .segments
                .last()
                .and_then(|segment| Name::from(&segment.ident).matches::<Whole>(&Self::MANY))
                .map(|_| Self::Many),
            _ => None,
        }
    }

    fn named(name: &Name, selfish: Option<&Name>) -> Self {
        name.matches::<Whole>(&Self::OPTIONAL)
            .map(|_| Self::Optional)
            .or_else(|| name.matches::<Whole>(&Self::FALLIBLE).map(|_| Self::Fallible))
            .or_else(|| name.matches::<Whole>(&Self::MANY).map(|_| Self::Many))
            .unwrap_or_else(|| MethodShape::itself(name, selfish).map_or(Self::Value, |_| Self::Itself))
    }
}

impl fmt::Display for Receiver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => formatter.write_str(""),
            Self::Owned => formatter.write_str("self"),
            Self::Shared => formatter.write_str("&self"),
            Self::Mutable => formatter.write_str("&mut self"),
        }
    }
}

impl fmt::Display for Taken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Itself => formatter.write_str("Self"),
            Self::Other => formatter.write_str("_"),
        }
    }
}

impl fmt::Display for Returned {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => formatter.write_str(""),
            Self::Itself => formatter.write_str(" -> Self"),
            Self::Optional => formatter.write_str(" -> Option"),
            Self::Fallible => formatter.write_str(" -> Result"),
            Self::Many => formatter.write_str(" -> many"),
            Self::Value => formatter.write_str(" -> _"),
        }
    }
}

impl fmt::Display for MethodShape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let receiver = (self.receiver != Receiver::Absent).then(|| self.receiver.to_string());
        let taken = self.takes.iter().map(ToString::to_string);
        let inputs: Vec<String> = receiver.into_iter().chain(taken).collect();
        write!(formatter, "fn({}){}", inputs.join(", "), self.returned)
    }
}
