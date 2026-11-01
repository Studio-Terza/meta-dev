use std::fmt;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Named {
    name: Option<String>,
    file: Option<String>,
    replacement: Option<String>,
}

impl Named {
    const UNKNOWN_FILE: &'static str = "?";
}

impl fmt::Display for Named {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name.as_deref().filter(|name| !name.is_empty()) {
            Some(name) => formatter.write_str(name),
            None => write!(
                formatter,
                "{}: {}",
                self.file.as_deref().unwrap_or(Self::UNKNOWN_FILE),
                self.replacement.as_deref().unwrap_or_default()
            ),
        }
    }
}
