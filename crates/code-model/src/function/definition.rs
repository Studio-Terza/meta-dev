use std::fmt;

use body_lifts::{Body, Branching};
use lint_layout::{Casing, LineNumber, Name, Whole, Word};

use crate::boundary::{CallOrder, ParamIndex};
use crate::type_ref::{Named, TypeRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receiving {
    Owner,
    Bound(TypeRef),
    Call(CallOrder),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    Method { receiver: Receiving, name: Name },
    Path { owner: Option<Name>, name: Name },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallOwner<'name> {
    Type(&'name Name),
    Free,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    shape: Body,
    branching: Branching,
    invocations: Vec<Invocation>,
    repetitions: Vec<Repetition>,
}

impl Definition {
    #[must_use]
    pub fn new(shape: Body, branching: Branching, invocations: Vec<Invocation>) -> Definition {
        Definition { shape, branching, invocations, repetitions: Vec::new() }
    }

    #[must_use]
    pub fn repeated(self, repetitions: Vec<Repetition>) -> Definition {
        Definition { repetitions, ..self }
    }

    #[must_use]
    pub fn repetitions(&self) -> &[Repetition] {
        &self.repetitions
    }

    #[must_use]
    pub fn shape(&self) -> Body {
        self.shape
    }

    #[must_use]
    pub fn branching(&self) -> Branching {
        self.branching
    }

    #[must_use]
    pub fn invocations(&self) -> &[Invocation] {
        &self.invocations
    }
}

impl Invocation {
    #[must_use]
    pub fn method(receiver: Receiving, name: Name) -> Invocation {
        Invocation::Method { receiver, name }
    }

    #[must_use]
    pub fn owner<'name>(&'name self, caller: Option<&'name Name>, earlier: &[Option<&'name Name>]) -> CallOwner<'name> {
        match self {
            Invocation::Method { receiver, .. } => receiver.owner(caller, earlier),
            Invocation::Path { owner, .. } => owner
                .as_ref()
                .filter(|candidate| candidate.casing() != Casing::Lower)
                .map_or(CallOwner::Free, |type_name| CallOwner::typed(type_name, caller)),
        }
    }
}

impl Receiving {
    fn owner<'name>(&'name self, caller: Option<&'name Name>, earlier: &[Option<&'name Name>]) -> CallOwner<'name> {
        match self {
            Receiving::Owner => CallOwner::of_caller(caller),
            Receiving::Bound(ty) => ty.named().map_or(CallOwner::Unknown, |named| CallOwner::bound(named, caller)),
            Receiving::Call(order) => order.typed_in(earlier).map_or(CallOwner::Unknown, CallOwner::Type),
            Receiving::Unknown => CallOwner::Unknown,
        }
    }
}

impl<'name> CallOwner<'name> {
    const OWN_TYPE: Word<'static> = Word::new("Self");

    fn of_caller(caller: Option<&'name Name>) -> CallOwner<'name> {
        caller.map_or(CallOwner::Unknown, CallOwner::Type)
    }

    fn typed(owner: &'name Name, caller: Option<&'name Name>) -> CallOwner<'name> {
        owner.matches::<Whole>(&[Self::OWN_TYPE]).map_or(CallOwner::Type(owner), |_| Self::of_caller(caller))
    }

    fn bound(named: &'name Named<crate::sited::Sited>, caller: Option<&'name Name>) -> CallOwner<'name> {
        named.last_spelled().map_or_else(|| Self::of_caller(caller), |owner| Self::typed(owner, caller))
    }

    #[must_use]
    pub fn admits(self, subject: Option<&Name>) -> Option<CallOwner<'name>> {
        match self {
            CallOwner::Type(owner) => subject.filter(|candidate| *candidate == owner).map(|_| self),
            CallOwner::Free => subject.map_or(Some(self), |_| None),
            CallOwner::Unknown => None,
        }
    }
}

impl AsRef<Name> for Invocation {
    fn as_ref(&self) -> &Name {
        match self {
            Invocation::Method { name, .. } | Invocation::Path { name, .. } => name,
        }
    }
}

impl fmt::Display for Receiving {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Receiving::Owner => formatter.write_str("self"),
            Receiving::Bound(ty) => write!(formatter, "{}", ty.brief()),
            Receiving::Call(order) => write!(formatter, "{order}"),
            Receiving::Unknown => formatter.write_str("?"),
        }
    }
}

impl fmt::Display for Invocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invocation::Method { receiver, name } => write!(formatter, "{receiver}.{name}"),
            Invocation::Path { owner: Some(owner), name } => write!(formatter, "{owner}::{name}"),
            Invocation::Path { owner: None, name } => write!(formatter, "{name}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Looping {
    For,
    While,
    Bare,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Repeating {
    Closure(Name),
    Loop(Looping),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reached {
    Parameter { index: ParamIndex, name: Name },
    Field(Name),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repetition {
    reach: Reached,
    line: LineNumber,
    repeating: Repeating,
    method: Option<Name>,
}

impl Repetition {
    #[must_use]
    pub fn new(reach: Reached, line: LineNumber, repeating: Repeating, method: Option<Name>) -> Repetition {
        Repetition { reach, line, repeating, method }
    }

    #[must_use]
    pub fn reach(&self) -> &Reached {
        &self.reach
    }

    #[must_use]
    pub fn repeating(&self) -> &Repeating {
        &self.repeating
    }

    #[must_use]
    pub fn method(&self) -> Option<&Name> {
        self.method.as_ref()
    }
}

impl AsRef<LineNumber> for Repetition {
    fn as_ref(&self) -> &LineNumber {
        &self.line
    }
}

impl Reached {
    #[must_use]
    pub fn index(&self) -> Option<ParamIndex> {
        match self {
            Reached::Parameter { index, .. } => Some(*index),
            Reached::Field(_) => None,
        }
    }
}

impl AsRef<Name> for Reached {
    fn as_ref(&self) -> &Name {
        match self {
            Reached::Parameter { name, .. } | Reached::Field(name) => name,
        }
    }
}

impl fmt::Display for Looping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Looping::For => formatter.write_str("for"),
            Looping::While => formatter.write_str("while"),
            Looping::Bare => formatter.write_str("loop"),
        }
    }
}

impl fmt::Display for Repeating {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Repeating::Closure(adaptor) => write!(formatter, "the closure passed to {adaptor}, once per element"),
            Repeating::Loop(looping) => write!(formatter, "a {looping} loop body, once per iteration"),
        }
    }
}
