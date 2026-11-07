use std::fmt;

use crate::language::Language;
use crate::place::Place;

#[derive(Clone, Copy, Debug)]
pub struct Finding {
    language: Language,
    place: Place,
}

impl Finding {
    pub(crate) const fn new(language: Language, place: Place) -> Self {
        Self { language, place }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.language, self.place)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> lint_core::Fix {
        lint_core::Fix::from(self.language.fix())
    }
}
