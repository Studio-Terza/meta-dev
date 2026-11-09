use serde::Deserialize;

use crate::format::Format;

#[derive(Clone, Debug, Deserialize)]
pub struct LexerTable {
    format: Format,
}

impl AsRef<Format> for LexerTable {
    fn as_ref(&self) -> &Format {
        &self.format
    }
}
