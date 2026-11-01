use std::fmt;

use crate::language::Language;

#[derive(Clone, Copy, Debug)]
pub struct Finding {
    language: Language,
}

impl Finding {
    pub fn new(language: Language) -> Finding {
        Finding { language }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} source outside assets; {}", self.language.name(), self.language.fix())
    }
}
