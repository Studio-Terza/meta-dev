use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Column<Name> {
    name: Name,
    type_info: Storage,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "Option<bool>")]
pub enum Nullable {
    Never,
    Possible,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Storage {
    Integer,
    Int4,
    Int64,
    Real,
    Float,
    Text,
    Blob,
    #[serde(other)]
    Unmapped,
}

#[derive(Debug)]
pub struct Described<Name> {
    column: Column<Name>,
    nullable: Nullable,
}

impl<Name> Column<Name> {
    pub(super) fn described(self, nullable: Nullable) -> Described<Name> {
        Described { column: self, nullable }
    }
}

impl<Name> Described<Name> {
    #[must_use]
    pub fn storage(&self) -> Storage {
        self.column.type_info
    }

    #[must_use]
    pub fn nullable(&self) -> Nullable {
        self.nullable
    }
}

impl<Name> AsRef<Name> for Described<Name> {
    fn as_ref(&self) -> &Name {
        &self.column.name
    }
}

impl Nullable {
    #[must_use]
    pub fn possible(self) -> Option<()> {
        match self {
            Nullable::Never => None,
            Nullable::Possible => Some(()),
        }
    }
}

impl Storage {
    #[must_use]
    pub fn spelled(self) -> Option<&'static str> {
        match self {
            Storage::Integer | Storage::Int4 | Storage::Int64 => Some("i64"),
            Storage::Real | Storage::Float => Some("f64"),
            Storage::Text => Some("::std::string::String"),
            Storage::Blob => Some("::std::vec::Vec<u8>"),
            Storage::Unmapped => None,
        }
    }
}

impl From<Option<bool>> for Nullable {
    fn from(described: Option<bool>) -> Nullable {
        described
            .and_then(|nullable| nullable.then_some(Nullable::Possible).or(Some(Nullable::Never)))
            .unwrap_or(Nullable::Possible)
    }
}
