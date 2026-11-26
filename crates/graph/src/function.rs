pub(crate) mod definition;
mod shape;

use std::fmt;
use std::ops::Range;

use lint_layout::Name;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::vec;
use newtype::proptest::strategy::Strategy;

use crate::item::Exposure;
use crate::region::{Region, Seat};
use crate::type_ref::TypeRef;

use definition::Definition;
pub use shape::{MethodShape, Returned, Taken};

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Free,
    Inherent,
    TraitDecl,
    TraitImpl,
    Nested,
    Foreign,
}

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evaluation {
    Runtime,
    Constant,
}

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Test,
    Main,
    Called,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
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

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
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

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mutation {
    Absent,
    ThroughSelf,
    ThroughParameter,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(generate = Parameter::sampled)]
pub struct Parameter {
    name: Name,
    bounds: Vec<TypeRef>,
}

impl Parameter {
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = Parameter> {
        Strategy::prop_map((any::<Name>(), vec(any::<TypeRef>(), Parameter::SAMPLED)), |(name, bounds)| {
            Parameter::new(name, bounds)
        })
    }

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

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(generate = Signature::sampled)]
pub struct Signature {
    inputs: Vec<Input>,
    output: Output,
    constraints: Vec<TypeRef>,
    parameters: Vec<Parameter>,
    mutation: Mutation,
    mutated: Mutated,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mutated(Vec<TypeRef>);

impl AsRef<[TypeRef]> for Mutated {
    fn as_ref(&self) -> &[TypeRef] {
        &self.0
    }
}

impl AsRef<Mutated> for Signature {
    fn as_ref(&self) -> &Mutated {
        &self.mutated
    }
}

impl From<Output> for Signature {
    fn from(output: Output) -> Signature {
        Signature {
            inputs: Vec::new(),
            output,
            constraints: Vec::new(),
            parameters: Vec::new(),
            mutation: Mutation::Absent,
            mutated: Mutated::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monomorphic {
    inputs: Vec<Input>,
    output: Output,
    constraints: Vec<TypeRef>,
}

impl Monomorphic {
    #[must_use]
    pub fn new(inputs: Vec<Input>, output: Output, constraints: Vec<TypeRef>) -> Monomorphic {
        Monomorphic { inputs, output, constraints }
    }

    #[must_use]
    pub fn generic(self, parameters: Vec<Parameter>, mutation: Mutation, mutated: Vec<TypeRef>) -> Signature {
        let Monomorphic { inputs, output, constraints } = self;
        Signature { inputs, output, constraints, parameters, mutation, mutated: Mutated(mutated) }
    }
}

impl Signature {
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = Signature> {
        let listed = || vec(any::<TypeRef>(), Signature::SAMPLED);
        Strategy::prop_map(
            (
                vec(any::<Input>(), Signature::SAMPLED),
                any::<Output>(),
                listed(),
                vec(any::<Parameter>(), Signature::SAMPLED),
                any::<Mutation>(),
                listed(),
            ),
            |(inputs, output, constraints, parameters, mutation, mutated)| {
                Monomorphic::new(inputs, output, constraints).generic(parameters, mutation, mutated)
            },
        )
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

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Function {
    name: Name,
    owner: Owner,
    entry: Entry,
    signature: Signature,
    seat: Seat,
    definition: Option<Definition>,
    exposure: Exposure,
    evaluation: Evaluation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signed {
    name: Name,
    owner: Owner,
    entry: Entry,
    signature: Signature,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seated {
    signed: Signed,
    seat: Seat,
    exposure: Exposure,
}

impl Signed {
    #[must_use]
    pub fn new(name: Name, owner: Owner, entry: Entry, signature: Signature) -> Signed {
        Signed { name, owner, entry, signature }
    }

    #[must_use]
    pub fn seated(self, seat: Seat, exposure: Exposure) -> Seated {
        Seated { signed: self, seat, exposure }
    }
}

impl Seated {
    #[must_use]
    pub fn defined(self, evaluation: Evaluation, definition: Option<Definition>) -> Function {
        let Seated { signed: Signed { name, owner, entry, signature }, seat, exposure } = self;
        Function { name, owner, entry, signature, seat, definition, exposure, evaluation }
    }
}

impl Function {
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

impl AsRef<Exposure> for Function {
    fn as_ref(&self) -> &Exposure {
        &self.exposure
    }
}

impl AsRef<Evaluation> for Function {
    fn as_ref(&self) -> &Evaluation {
        &self.evaluation
    }
}
