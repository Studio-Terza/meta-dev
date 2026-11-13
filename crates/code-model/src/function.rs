pub(crate) mod definition;
mod shape;

use std::fmt;

use lint_layout::Name;

use crate::region::{Region, Seat};
use crate::type_ref::TypeRef;

use definition::Definition;
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
    #[must_use]
    pub fn typed(&self) -> Option<&TypeRef> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mutation {
    Absent,
    ThroughSelf,
    ThroughParameter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    name: Name,
    bounds: Vec<TypeRef>,
}

impl Parameter {
    #[must_use]
    pub fn new(name: Name, bounds: Vec<TypeRef>) -> Parameter {
        Parameter { name, bounds }
    }

    #[must_use]
    pub fn bounds(&self) -> &[TypeRef] {
        &self.bounds
    }
}

impl AsRef<Name> for Parameter {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    inputs: Vec<Input>,
    output: Output,
    constraints: Vec<TypeRef>,
    parameters: Vec<Parameter>,
    mutation: Mutation,
}

impl From<Output> for Signature {
    fn from(output: Output) -> Signature {
        Signature {
            inputs: Vec::new(),
            output,
            constraints: Vec::new(),
            parameters: Vec::new(),
            mutation: Mutation::Absent,
        }
    }
}

impl Signature {
    #[must_use]
    pub fn new(inputs: Vec<Input>, output: Output, constraints: Vec<TypeRef>) -> Signature {
        Signature { inputs, output, constraints, parameters: Vec::new(), mutation: Mutation::Absent }
    }

    #[must_use]
    pub fn generic(self, parameters: Vec<Parameter>, mutation: Mutation) -> Signature {
        Signature { parameters, mutation, ..self }
    }

    #[must_use]
    pub fn mutation(&self) -> Mutation {
        self.mutation
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
    definition: Option<Definition>,
}

impl Function {
    #[must_use]
    pub fn new(name: Name, owner: Owner, entry: Entry, signature: Signature) -> Function {
        Function { name, owner, entry, signature, seat: Seat::new(Region::SHIPPED, None), definition: None }
    }

    #[must_use]
    pub fn seated(self, seat: Seat) -> Function {
        Function { seat, ..self }
    }

    #[must_use]
    pub fn defined(self, definition: Option<Definition>) -> Function {
        Function { definition, ..self }
    }

    #[must_use]
    pub fn definition(&self) -> Option<&Definition> {
        self.definition.as_ref()
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

impl AsRef<[Parameter]> for Signature {
    fn as_ref(&self) -> &[Parameter] {
        &self.parameters
    }
}
