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
    pub fn of_signature(signature: &Signature, selfish: Option<&Name>) -> MethodShape {
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
        MethodShape { receiver, takes, returned }
    }

    #[must_use]
    pub fn receiver(&self) -> Receiver {
        self.receiver
    }

    #[must_use]
    pub fn takes(&self) -> &[Taken] {
        &self.takes
    }

    #[must_use]
    pub fn returned(&self) -> Returned {
        self.returned
    }

    fn itself(name: &Name, selfish: Option<&Name>) -> Option<Taken> {
        name.matches::<Whole>(&Self::SELF)
            .map(|_| Taken::Itself)
            .or_else(|| selfish.filter(|own| *own == name).map(|_| Taken::Itself))
    }
}

impl From<&syn::Receiver> for Receiver {
    fn from(found: &syn::Receiver) -> Receiver {
        found
            .reference
            .as_ref()
            .map_or(Receiver::Owned, |_| found.mutability.map_or(Receiver::Shared, |_| Receiver::Mutable))
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

    fn of_type(found: &Type, selfish: Option<&Name>) -> Returned {
        match found {
            Type::Reference(reference) => Self::of_type(&reference.elem, selfish),
            Type::Paren(paren) => Self::of_type(&paren.elem, selfish),
            Type::Tuple(tuple) => tuple.elems.first().map_or(Returned::Nothing, |_| Returned::Value),
            Type::Slice(_) | Type::Array(_) => Returned::Many,
            Type::ImplTrait(bounds) => bounds.bounds.iter().find_map(Self::bounded).unwrap_or(Returned::Value),
            Type::Path(_) => Spelled::last_of(found).map_or(Returned::Value, |name| Self::named(&name, selfish)),
            _ => Returned::Value,
        }
    }

    fn bounded(bound: &TypeParamBound) -> Option<Returned> {
        match bound {
            TypeParamBound::Trait(found) => found
                .path
                .segments
                .last()
                .and_then(|segment| Name::from(&segment.ident).matches::<Whole>(&Self::MANY))
                .map(|_| Returned::Many),
            _ => None,
        }
    }

    fn named(name: &Name, selfish: Option<&Name>) -> Returned {
        name.matches::<Whole>(&Self::OPTIONAL)
            .map(|_| Returned::Optional)
            .or_else(|| name.matches::<Whole>(&Self::FALLIBLE).map(|_| Returned::Fallible))
            .or_else(|| name.matches::<Whole>(&Self::MANY).map(|_| Returned::Many))
            .unwrap_or_else(|| MethodShape::itself(name, selfish).map_or(Returned::Value, |_| Returned::Itself))
    }
}

impl fmt::Display for Receiver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Receiver::Absent => formatter.write_str(""),
            Receiver::Owned => formatter.write_str("self"),
            Receiver::Shared => formatter.write_str("&self"),
            Receiver::Mutable => formatter.write_str("&mut self"),
        }
    }
}

impl fmt::Display for Taken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Taken::Itself => formatter.write_str("Self"),
            Taken::Other => formatter.write_str("_"),
        }
    }
}

impl fmt::Display for Returned {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Returned::Nothing => formatter.write_str(""),
            Returned::Itself => formatter.write_str(" -> Self"),
            Returned::Optional => formatter.write_str(" -> Option"),
            Returned::Fallible => formatter.write_str(" -> Result"),
            Returned::Many => formatter.write_str(" -> many"),
            Returned::Value => formatter.write_str(" -> _"),
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
