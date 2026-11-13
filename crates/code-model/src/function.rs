mod shape;

use std::fmt;

use lint_layout::Name;

use crate::region::{Region, Seat};
use crate::type_ref::TypeRef;

pub use shape::{MethodShape, Returned, Taken};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Free,
    Inherent,
    TraitDecl,
    TraitImpl,
    Nested,
    Foreign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Test,
    Main,
    Called,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Implicit,
    Written(TypeRef),
}

impl Output {
    #[must_use]
    pub fn written(&self) -> Option<&TypeRef> {
        match self {
            Output::Written(ty) => Some(ty),
            Output::Implicit => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Receiver {
    Owned,
    Shared,
    Mutable,
}

impl fmt::Display for Receiver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Receiver::Owned => formatter.write_str("self"),
            Receiver::Shared => formatter.write_str("&self"),
            Receiver::Mutable => formatter.write_str("&mut self"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Receiver(Receiver),
    Typed(TypeRef),
}

impl Input {
    fn typed(&self) -> Option<&TypeRef> {
        match self {
            Input::Typed(ty) => Some(ty),
            Input::Receiver(_) => None,
        }
    }

    #[must_use]
    pub fn receiver(&self) -> Option<Receiver> {
        match self {
            Input::Receiver(receiver) => Some(*receiver),
            Input::Typed(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    inputs: Vec<Input>,
    output: Output,
    constraints: Vec<TypeRef>,
}

impl From<Output> for Signature {
    fn from(output: Output) -> Signature {
        Signature { inputs: Vec::new(), output, constraints: Vec::new() }
    }
}

impl Signature {
    #[must_use]
    pub fn new(inputs: Vec<Input>, output: Output, constraints: Vec<TypeRef>) -> Signature {
        Signature { inputs, output, constraints }
    }

    #[must_use]
    pub fn output(&self) -> &Output {
        &self.output
    }

    #[must_use]
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn written(&self) -> impl Iterator<Item = &TypeRef> {
        self.constraints.iter().chain(self.inputs.iter().filter_map(Input::typed)).chain(self.output.written())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    name: Name,
    owner: Owner,
    entry: Entry,
    signature: Signature,
    seat: Seat,
}

impl Function {
    #[must_use]
    pub fn new(name: Name, owner: Owner, entry: Entry, signature: Signature) -> Function {
        Function { name, owner, entry, signature, seat: Seat::new(Region::SHIPPED, None) }
    }

    #[must_use]
    pub fn seated(self, seat: Seat) -> Function {
        Function { name: self.name, owner: self.owner, entry: self.entry, signature: self.signature, seat }
    }

    #[must_use]
    pub fn signed_here(&self) -> Option<&Signature> {
        match self.owner {
            Owner::Free | Owner::Inherent | Owner::TraitDecl | Owner::Nested => Some(&self.signature),
            Owner::TraitImpl | Owner::Foreign => None,
        }
    }

    #[must_use]
    pub fn called(&self) -> Option<&Function> {
        match self.entry {
            Entry::Called => Some(self),
            Entry::Test | Entry::Main => None,
        }
    }

    #[must_use]
    pub fn entry(&self) -> Entry {
        self.entry
    }

    #[must_use]
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    #[must_use]
    pub fn owner(&self) -> Owner {
        self.owner
    }
}

impl AsRef<Seat> for Function {
    fn as_ref(&self) -> &Seat {
        &self.seat
    }
}

impl AsRef<Region> for Function {
    fn as_ref(&self) -> &Region {
        self.seat.as_ref()
    }
}

impl AsRef<Name> for Function {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
