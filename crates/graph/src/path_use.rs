use lint_layout::PathTail;

use crate::external::External;
use crate::item_path::ItemPath;
use crate::region::{Region, Seat};
use crate::resolved::Resolved;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mention {
    Expression,
    Invoked,
    InMacro,
    Interpolated,
}

impl Mention {
    pub(crate) fn interpolated(self) -> Option<Mention> {
        match self {
            Mention::Interpolated => Some(self),
            Mention::Expression | Mention::Invoked | Mention::InMacro => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathUse {
    written: ItemPath,
    resolved: Resolved,
    mention: Mention,
    seat: Seat,
}

impl PathUse {
    #[must_use]
    pub fn new(written: ItemPath, resolved: Resolved, mention: Mention, seat: Seat) -> PathUse {
        PathUse { written, resolved, mention, seat }
    }

    #[must_use]
    pub fn written(&self) -> &ItemPath {
        &self.written
    }

    #[must_use]
    pub fn resolved(&self) -> &Resolved {
        &self.resolved
    }

    #[must_use]
    pub fn mention(&self) -> Mention {
        self.mention
    }

    #[must_use]
    pub fn spelling(&self, tail: PathTail<'_>) -> Option<&ItemPath> {
        let resolved = self.resolved.std().or_else(|| self.resolved.external().map(External::path));
        resolved.into_iter().chain([&self.written]).find_map(|spelled| spelled.ends_with(tail))
    }
}

impl AsRef<Seat> for PathUse {
    fn as_ref(&self) -> &Seat {
        &self.seat
    }
}

impl AsRef<Region> for PathUse {
    fn as_ref(&self) -> &Region {
        self.seat.as_ref()
    }
}
