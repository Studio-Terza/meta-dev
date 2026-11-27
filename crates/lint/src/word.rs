use syn::{PathSegment, Type, TypePath};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Word(pub &'static str);

impl Word {
    pub(crate) fn last(self, ty: &Type) -> Option<&PathSegment> {
        match ty {
            Type::Path(TypePath { path, .. }) => path.segments.last().filter(|segment| segment.ident == self),
            _ => None,
        }
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        self.0
    }
}
