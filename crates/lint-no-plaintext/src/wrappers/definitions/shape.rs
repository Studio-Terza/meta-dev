use syn::{Ident, Type};

use lint_layout::Name;

use crate::text_mention::TextMention;
use crate::wrappers::Wrappers;

pub struct Shape<'file> {
    name: &'file Ident,
    field: Option<&'file Type>,
}

impl<'file> Shape<'file> {
    pub fn new(name: &'file Ident, field: Option<&'file Type>) -> Shape<'file> {
        Shape { name, field }
    }

    pub fn answers_to(&self, name: &Name) -> Option<&Shape<'file>> {
        (*name == *self.name).then_some(self)
    }

    pub fn holds_text(&self, settled: &Wrappers) -> Option<&'file Ident> {
        self.field.and_then(|field| TextMention::in_type(field, settled))
    }
}
