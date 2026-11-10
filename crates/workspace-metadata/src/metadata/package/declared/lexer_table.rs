use serde::Deserialize;

use crate::format::Format;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexerTable {
    format: Format,
}

impl AsRef<Format> for LexerTable {
    fn as_ref(&self) -> &Format {
        &self.format
    }
}
