use syn::Type;

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

    pub fn answers_to(&self, name: &Name) -> bool {
        self.name == *name
    }

    pub fn holds_text(&self, settled: &Wrappers) -> bool {
        self.field.as_ref().is_some_and(|field| TextMention::in_type(field, settled))
    }
}
