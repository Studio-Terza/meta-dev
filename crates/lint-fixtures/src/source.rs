use std::path::Path;

use newtype::Newtype;

use crate::located::Located;
use crate::text::Text;

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
    pub fn path(&self) -> &'static Path {
        Path::new(self.path.into_inner())
    }

    #[must_use]
    pub const fn text(&self) -> Text<'static> {
        self.text
    }
}
