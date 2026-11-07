use lint_layout::Name;

use crate::region::{Region, Seat};
use crate::type_ref::TypeRef;

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
            Self::Written(ty) => Some(ty),
            Self::Implicit => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Receiver,
    Typed(TypeRef),
}

impl Input {
    fn typed(&self) -> Option<&TypeRef> {
        match self {
            Self::Typed(ty) => Some(ty),
            Self::Receiver => None,
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
    fn from(output: Output) -> Self {
        Self { inputs: Vec::new(), output, constraints: Vec::new() }
    }
}

impl Signature {
    #[must_use]
    pub fn new(inputs: Vec<Input>, output: Output, constraints: Vec<TypeRef>) -> Self {
        Self { inputs, output, constraints }
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
    pub fn new(name: Name, owner: Owner, entry: Entry, signature: Signature) -> Self {
        Self { name, owner, entry, signature, seat: Seat::new(Region::SHIPPED, None) }
    }

    #[must_use]
    pub fn seated(self, seat: Seat) -> Self {
        Self { name: self.name, owner: self.owner, entry: self.entry, signature: self.signature, seat }
    }

    #[must_use]
    pub fn signed_here(&self) -> Option<&Signature> {
        match self.owner {
            Owner::Free | Owner::Inherent | Owner::TraitDecl | Owner::Nested => Some(&self.signature),
            Owner::TraitImpl | Owner::Foreign => None,
        }
    }

    #[must_use]
    pub fn called(&self) -> Option<&Self> {
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
