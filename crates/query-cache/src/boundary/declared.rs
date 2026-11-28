use invariants::Ignoring;
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::Ignored;

#[derive(Debug, Deserialize, Serialize)]
pub struct Column<Name> {
    name: Name,
    type_info: Storage,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(from = "Option<bool>", into = "Option<bool>")]
pub enum Nullable {
    Never,
    Possible,
    Unknown,
}

#[derive(EnumString, Clone, Copy, Debug, Deserialize, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Storage {
    #[strum(serialize = "INTEGER", serialize = "INT")]
    Integer,
    Real,
    Text,
    Blob,
    #[serde(rename = "Null", other)]
    #[strum(disabled)]
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
    pub fn new(name: Name, declared: Option<&str>, nullable: Option<bool>) -> Described<Name> {
        let type_info = declared
            .and_then(|spelled| spelled.parse::<Storage>().ignoring(Ignored::Unmapped))
            .unwrap_or(Storage::Unmapped);
        Described { column: Column { name, type_info }, nullable: Nullable::from(nullable) }
    }

    pub(super) fn into_column(self) -> Column<Name> {
        self.column
    }

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
            Nullable::Possible | Nullable::Unknown => Some(()),
        }
    }
}

impl Storage {
    #[must_use]
    pub fn spelled(self) -> Option<&'static str> {
        match self {
            Storage::Integer => Some("i64"),
            Storage::Real => Some("f64"),
            Storage::Text => Some("::std::string::String"),
            Storage::Blob => Some("::std::vec::Vec<u8>"),
            Storage::Unmapped => None,
        }
    }
}

impl From<Option<bool>> for Nullable {
    fn from(described: Option<bool>) -> Nullable {
        match described {
            Some(false) => Nullable::Never,
            Some(true) => Nullable::Possible,
            None => Nullable::Unknown,
        }
    }
}

impl From<Nullable> for Option<bool> {
    fn from(nullable: Nullable) -> Option<bool> {
        match nullable {
            Nullable::Never => Some(false),
            Nullable::Possible => Some(true),
            Nullable::Unknown => None,
        }
    }
}
