use lint_layout::Name;

use crate::item_path::ItemPath;
use crate::region::{Region, Seat};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    Method(Name),
    Path(ItemPath),
    Macro(MacroCall),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroCall {
    written: ItemPath,
}

impl From<ItemPath> for MacroCall {
    fn from(written: ItemPath) -> Self {
        Self { written }
    }
}

impl AsRef<ItemPath> for MacroCall {
    fn as_ref(&self) -> &ItemPath {
        &self.written
    }
}

impl Callee {
    #[must_use]
    pub const fn method(&self) -> Option<&Name> {
        match self {
            Self::Method(name) => Some(name),
            Self::Path(_) | Self::Macro(_) => None,
        }
    }

    #[must_use]
    pub const fn path(&self) -> Option<&ItemPath> {
        match self {
            Self::Path(written) => Some(written),
            Self::Method(_) | Self::Macro(_) => None,
        }
    }

    #[must_use]
    pub const fn invoked(&self) -> Option<&MacroCall> {
        match self {
            Self::Macro(call) => Some(call),
            Self::Method(_) | Self::Path(_) => None,
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
    pub const fn new(callee: Callee, seat: Seat) -> Self {
        Self { callee, seat }
    }

    #[must_use]
    pub const fn callee(&self) -> &Callee {
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
