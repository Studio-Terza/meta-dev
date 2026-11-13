use std::fmt;

use lint_layout::{Name, Whole, Word, words};

use crate::region::Seat;
use crate::resolved::Resolved;
use crate::sited::Sited;
use crate::type_ref::{Named, Outline, TypeRef};

use super::{Function, Input, Receiver};

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
    receiver: Option<Receiver>,
    takes: Vec<Taken>,
    returned: Returned,
}

#[derive(Clone, Copy, Debug)]
struct Selfhood<'model> {
    subject: Option<&'model Resolved>,
}

impl MethodShape {
    #[must_use]
    pub fn new(receiver: Option<Receiver>, takes: Vec<Taken>, returned: Returned) -> MethodShape {
        MethodShape { receiver, takes, returned }
    }

    #[must_use]
    pub fn receiver(&self) -> Option<Receiver> {
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
}

impl Returned {
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
    pub fn named(name: &Name) -> Option<Returned> {
        name.matches::<Whole>(&Self::OPTIONAL)
            .map(|_| Returned::Optional)
            .or_else(|| name.matches::<Whole>(&Self::FALLIBLE).map(|_| Returned::Fallible))
            .or_else(|| name.matches::<Whole>(&Self::MANY).map(|_| Returned::Many))
            .or_else(|| name.matches::<Whole>(&Self::SELF).map(|_| Returned::Itself))
    }
}

impl Selfhood<'_> {
    fn own(self, named: &Named<Sited>) -> Option<Returned> {
        let resolved: &Resolved = named.as_ref();
        let subject = self.subject.filter(|subject| *subject == resolved).map(|_| Returned::Itself);
        subject.or_else(|| resolved.parameter().and_then(Returned::named).filter(|found| *found == Returned::Itself))
    }

    fn taken(self, ty: &TypeRef) -> Taken {
        ty.dereferenced().named().and_then(|named| self.own(named)).map_or(Taken::Other, |_| Taken::Itself)
    }

    fn std_named(named: &Named<Sited>) -> Option<Returned> {
        AsRef::<Resolved>::as_ref(named).std()?.last().and_then(Returned::named)
    }
}

impl Outline for Selfhood<'_> {
    type Kind = Returned;

    fn named(self, named: &Named<Sited>) -> Returned {
        Self::std_named(named).or_else(|| self.own(named)).unwrap_or(Returned::Value)
    }

    fn grouped(self, parts: &[TypeRef]) -> Returned {
        parts.first().map_or(Returned::Nothing, |_| Returned::Value)
    }

    fn sequence(self) -> Returned {
        Returned::Many
    }

    fn bounded(self, bounds: &[TypeRef]) -> Returned {
        bounds.iter().filter_map(TypeRef::named).find_map(Self::std_named).unwrap_or(Returned::Value)
    }

    fn other(self) -> Returned {
        Returned::Value
    }
}

impl From<&Function> for MethodShape {
    fn from(function: &Function) -> MethodShape {
        let signature = function.signature();
        let subject = AsRef::<Seat>::as_ref(function).subject().and_then(TypeRef::named).map(AsRef::as_ref);
        let selfhood = Selfhood { subject };
        MethodShape {
            receiver: signature.inputs().iter().find_map(Input::receiver),
            takes: signature.inputs().iter().filter_map(Input::typed).map(|ty| selfhood.taken(ty)).collect(),
            returned: signature.output().written().map_or(Returned::Nothing, |ty| ty.outlined(selfhood)),
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
        let receiver = self.receiver.map(|found| found.to_string());
        let taken = self.takes.iter().map(ToString::to_string);
        let inputs: Vec<String> = receiver.into_iter().chain(taken).collect();
        write!(formatter, "fn({}){}", inputs.join(", "), self.returned)
    }
}
