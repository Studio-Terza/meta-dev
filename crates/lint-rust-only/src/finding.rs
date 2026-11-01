use std::fmt;

use crate::language::Language;
use crate::place::Place;

#[derive(Clone, Copy, Debug)]
pub struct Finding {
    language: Language,
    place: Place,
}

impl Finding {
    pub(crate) fn new(language: Language, place: Place) -> Finding {
        Finding { language, place }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}; {}", self.language.name(), self.place, self.language.fix())
    }
}
