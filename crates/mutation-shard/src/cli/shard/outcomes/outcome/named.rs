use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use text_lines::OneLine;

#[derive(Debug, Default, Deserialize)]
pub struct Named {
    name: Option<OneLine>,
    file: Option<PathBuf>,
    replacement: Option<OneLine>,
}

impl Named {
    const UNKNOWN_FILE: &'static str = "?";

    fn given_name(&self) -> Option<&OneLine> {
        self.name.as_ref().filter(|name| AsRef::<str>::as_ref(name).chars().next().is_some())
    }
}

impl fmt::Display for Named {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.given_name() {
            Some(name) => name.fmt(formatter),
            None => write!(
                formatter,
                "{}: {}",
                self.file.as_deref().unwrap_or(Path::new(Self::UNKNOWN_FILE)).display(),
                self.replacement.as_ref().map(ToString::to_string).unwrap_or_default()
            ),
        }
    }
}
