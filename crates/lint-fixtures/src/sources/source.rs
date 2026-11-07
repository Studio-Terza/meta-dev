mod located;

use std::path::Path;

use newtype::Newtype;

use super::Text;
pub use located::Located;

#[derive(Debug)]
pub struct Source {
    path: Located,
    text: Text<'static>,
}

impl Source {
    #[must_use]
    pub const fn new(path: Located, text: Text<'static>) -> Self {
        Self { path, text }
    }

    #[must_use]
    pub fn path(&self) -> &'static Path {
        Path::new(self.path.into_inner())
    }

    #[must_use]
    pub const fn text(&self) -> Text<'static> {
        self.text
    }
}
