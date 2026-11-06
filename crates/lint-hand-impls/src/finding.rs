use std::fmt;

use code_model::Subject;
use lint_core::Fix;
use lint_layout::{Name, Word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Finding<'model> {
    derivable: Word<'static>,
    subject: &'model Subject,
}

impl<'model> Finding<'model> {
    pub(crate) fn new(derivable: Word<'static>, subject: &'model Subject) -> Finding<'model> {
        Finding { derivable, subject }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "derive it; answer a special need with a type (a redacting wrapper, field order, a key type, a named constructor)",
        )
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.subject.metavariable() {
            Some(name) => write!(formatter, "hand impl of {} for ${name}", self.derivable),
            None => write!(formatter, "hand impl of {} for {}", self.derivable, AsRef::<Name>::as_ref(self.subject)),
        }
    }
}
