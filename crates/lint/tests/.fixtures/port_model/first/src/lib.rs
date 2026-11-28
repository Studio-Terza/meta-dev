use serde::de::Error;

pub use serde;

struct Parsed(toml::Table);
