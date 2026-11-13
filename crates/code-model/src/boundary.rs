use lint_layout::Name;

use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(usize);

impl FileId {
    pub(crate) fn next<Row>(rows: &[Row]) -> FileId {
        FileId(rows.len())
    }

    pub(crate) fn within<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(usize);

impl ItemId {
    pub(crate) fn next<Row>(rows: &[Row]) -> ItemId {
        ItemId(rows.len())
    }

    pub(crate) fn index(self) -> usize {
        self.0
    }
}

impl TypeRef {
    #[must_use]
    pub fn brief(&self) -> Name {
        Name::from(self.innermost().to_string().as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallOrder(usize);

impl CallOrder {
    #[must_use]
    pub fn of<Row>(earlier: &[Row]) -> Option<CallOrder> {
        earlier.len().checked_sub(1).map(CallOrder)
    }

    #[must_use]
    pub fn typed_in<'name>(self, earlier: &[Option<&'name Name>]) -> Option<&'name Name> {
        earlier.get(self.0).copied().flatten()
    }
}
