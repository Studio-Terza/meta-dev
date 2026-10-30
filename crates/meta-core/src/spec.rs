use serde::de::DeserializeOwned;
use std::fmt;

/// A data file under `self/` that meta reads from main.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelfFile {
    Rules,
    EscapeKinds,
    Predictions,
}

impl SelfFile {
    pub fn path(self) -> &'static str {
        match self {
            Self::Rules => "self/rules.toml",
            Self::EscapeKinds => "self/escapes.toml",
            Self::Predictions => "self/predictions.toml",
        }
    }

    pub(crate) fn parse<T: DeserializeOwned>(self, text: &str) -> Result<T, Unreadable> {
        toml::from_str(text).map_err(|error| Unreadable { file: self, why: error.to_string() })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreadable {
    file: SelfFile,
    why: String,
}

impl fmt::Display for Unreadable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is unreadable: {}", self.file.path(), self.why)
    }
}
