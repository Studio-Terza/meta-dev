mod codec;
mod document;

pub use codec::{Toml, TomlFailure};
pub use document::{Document, Table, Value};
