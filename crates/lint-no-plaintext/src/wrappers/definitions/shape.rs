use syn::{Ident, Type};

use lint_layout::Name;

use crate::text_mention::TextMention;
use crate::wrappers::Wrappers;

pub struct Shape {
    name: Name,
    field: Option<Type>,
}

impl Shape {
    pub fn new(name: Name, field: Option<Type>) -> Shape {
        Shape { name, field }
    }

    pub fn answers_to(&self, name: &Name) -> Option<&Shape> {
        (self.name == *name).then_some(self)
    }

    pub fn holds_text(&self, settled: &Wrappers) -> Option<Ident> {
        self.field.as_ref().and_then(|field| TextMention::in_type(field, settled))
    }
}
