use lint_layout::Name;

use crate::item_path::ItemPath;
use crate::region::{Region, Seat};
use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Expression,
    Statement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathCall {
    written: ItemPath,
    owner: TypeRef,
}

impl PathCall {
    #[must_use]
    pub fn new(written: ItemPath, owner: TypeRef) -> PathCall {
        PathCall { written, owner }
    }
}

impl AsRef<ItemPath> for PathCall {
    fn as_ref(&self) -> &ItemPath {
        &self.written
    }
}

impl AsRef<TypeRef> for PathCall {
    fn as_ref(&self) -> &TypeRef {
        &self.owner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroCall {
    written: ItemPath,
    position: Position,
}

impl MacroCall {
    #[must_use]
    pub fn new(written: ItemPath, position: Position) -> MacroCall {
        MacroCall { written, position }
    }

    #[must_use]
    pub fn position(&self) -> Position {
        self.position
    }
}

impl AsRef<ItemPath> for MacroCall {
    fn as_ref(&self) -> &ItemPath {
        &self.written
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    Method(Name),
    Path(PathCall),
    Macro(MacroCall),
}

impl Callee {
    #[must_use]
    pub fn method(&self) -> Option<&Name> {
        match self {
            Callee::Method(name) => Some(name),
            Callee::Path(_) | Callee::Macro(_) => None,
        }
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathCall> {
        match self {
            Callee::Path(call) => Some(call),
            Callee::Method(_) | Callee::Macro(_) => None,
        }
    }

    #[must_use]
    pub fn invoked(&self) -> Option<&MacroCall> {
        match self {
            Callee::Macro(call) => Some(call),
            Callee::Method(_) | Callee::Path(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    callee: Callee,
    seat: Seat,
}

impl Call {
    #[must_use]
    pub fn new(callee: Callee, seat: Seat) -> Call {
        Call { callee, seat }
    }

    #[must_use]
    pub fn callee(&self) -> &Callee {
        &self.callee
    }
}

impl AsRef<Seat> for Call {
    fn as_ref(&self) -> &Seat {
        &self.seat
    }
}

impl AsRef<Region> for Call {
    fn as_ref(&self) -> &Region {
        self.seat.as_ref()
    }
}
