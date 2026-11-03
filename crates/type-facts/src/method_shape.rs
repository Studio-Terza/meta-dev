use std::fmt;

use syn::{FnArg, ReturnType, Signature, Type, TypeParamBound};

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

    #[must_use]
    pub fn of_signature(signature: &Signature, selfish: Option<&Name>) -> MethodShape {
        let receiver =
            signature.receiver().map_or(Receiver::Absent, |found| match (&found.reference, &found.mutability) {
                (Some(_), Some(_)) => Receiver::Mutable,
                (Some(_), None) => Receiver::Shared,
                (None, _) => Receiver::Owned,
            });
        let takes = signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => Some(typed),
                FnArg::Receiver(_) => None,
            })
            .map(|typed| {
                Spelled::last_of(&typed.ty).and_then(|name| Self::itself(&name, selfish)).unwrap_or(Taken::Other)
            })
            .collect();
        let returned = match &signature.output {
            ReturnType::Default => Returned::Nothing,
            ReturnType::Type(_, found) => Self::returning(found, selfish),
        };
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

    fn returning(found: &Type, selfish: Option<&Name>) -> Returned {
        match found {
            Type::Reference(reference) => Self::returning(&reference.elem, selfish),
            Type::Paren(paren) => Self::returning(&paren.elem, selfish),
            Type::Tuple(tuple) if tuple.elems.is_empty() => Returned::Nothing,
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
        match name {
            _ if name.matches::<Whole>(&Self::OPTIONAL).is_some() => Returned::Optional,
            _ if name.matches::<Whole>(&Self::FALLIBLE).is_some() => Returned::Fallible,
            _ if name.matches::<Whole>(&Self::MANY).is_some() => Returned::Many,
            _ => Self::itself(name, selfish).map_or(Returned::Value, |_| Returned::Itself),
        }
    }
}

impl fmt::Display for Receiver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Receiver::Absent => "",
            Receiver::Owned => "self",
            Receiver::Shared => "&self",
            Receiver::Mutable => "&mut self",
        })
    }
}

impl fmt::Display for Taken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Taken::Itself => "Self",
            Taken::Other => "_",
        })
    }
}

impl fmt::Display for Returned {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Returned::Nothing => "",
            Returned::Itself => " -> Self",
            Returned::Optional => " -> Option",
            Returned::Fallible => " -> Result",
            Returned::Many => " -> many",
            Returned::Value => " -> _",
        })
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
