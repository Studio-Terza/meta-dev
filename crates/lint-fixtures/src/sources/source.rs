mod located;

use std::path::Path;

use newtype::Wrapping;

use super::Text;
pub use located::Located;

#[derive(Debug)]
pub struct Source {
    path: Located,
    text: Text<'static>,
}

impl Source {
    #[must_use]
    pub const fn new(path: Located, text: Text<'static>) -> Source {
        Source { path, text }
    }

    #[must_use]
    pub const fn text(&self) -> Text<'static> {
        self.text
    }
}

impl AsRef<Path> for Source {
    fn as_ref(&self) -> &Path {
        Path::new(self.path.into_inner())
    }
}
