use std::path::Path;

#[derive(Debug)]
pub struct Source {
    path: &'static str,
    text: &'static str,
}

impl Source {
    #[must_use]
    pub const fn new(path: &'static str, text: &'static str) -> Source {
        Source { path, text }
    }

    #[must_use]
    pub fn path(&self) -> &'static Path {
        Path::new(self.path)
    }

    #[must_use]
    pub const fn text(&self) -> &'static str {
        self.text
    }
}
