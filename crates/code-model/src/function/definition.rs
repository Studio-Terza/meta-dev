use body_lifts::{Body, Branching};
use lint_layout::Name;

use crate::boundary::CallOrder;
use crate::item_path::ItemPath;
use crate::type_ref::TypeRef;

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
    Path(ItemPath),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    shape: Body,
    branching: Branching,
    invocations: Vec<Invocation>,
}

impl Definition {
    #[must_use]
    pub fn new(shape: Body, branching: Branching, invocations: Vec<Invocation>) -> Definition {
        Definition { shape, branching, invocations }
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
}
