use std::fmt;

use lint_layout::Name;
use newtype::Arbitrary;

use crate::item_path::ItemPath;
use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(usize);

impl FileId {
    pub fn every<Row>(rows: &[Row]) -> impl Iterator<Item = FileId> + use<Row> {
        (0..rows.len()).map(FileId)
    }

    pub(crate) fn within<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(from = ItemId)]
pub struct ItemId(usize);

impl ItemId {
    pub(crate) fn numbered<Row>(scopes: &[Vec<Option<Row>>]) -> Vec<Vec<Option<ItemId>>> {
        let present = scopes.iter().flatten().map(Option::as_ref);
        let flat: Vec<Option<ItemId>> = present
            .scan(0, |next: &mut usize, declared| {
                let issued = declared.map(|_| ItemId(*next));
                *next += issued.map_or(0, |_| 1);
                Some(issued)
            })
            .collect();
        scopes
            .iter()
            .scan(0, |start: &mut usize, scope| {
                let end = *start + scope.len();
                let group = flat.get(*start..end).map_or_default(<[Option<ItemId>]>::to_vec);
                *start = end;
                Some(group)
            })
            .collect()
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
    pub fn latest<Row>(earlier: &[Row]) -> Option<CallOrder> {
        earlier.len().checked_sub(1).map(CallOrder)
    }

    #[must_use]
    pub fn typed_in<'name>(self, earlier: &[Option<&'name Name>]) -> Option<&'name Name> {
        earlier.get(self.0).copied().flatten()
    }
}

impl fmt::Display for CallOrder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}

impl fmt::Display for ItemPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = AsRef::<[Name]>::as_ref(self).iter().map(ToString::to_string).collect();
        formatter.write_str(&segments.join("::"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParamIndex(usize);

impl From<usize> for ParamIndex {
    fn from(index: usize) -> ParamIndex {
        ParamIndex(index)
    }
}

impl fmt::Display for ParamIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl ParamIndex {
    #[must_use]
    pub fn entry<Item>(self, items: &[Item]) -> Option<&Item> {
        items.get(self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote(String);

impl From<String> for Quote {
    fn from(written: String) -> Quote {
        Quote(written)
    }
}

impl AsRef<str> for Quote {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralValue(String);

impl From<&str> for LiteralValue {
    fn from(text: &str) -> LiteralValue {
        LiteralValue(text.to_owned())
    }
}
