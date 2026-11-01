use std::fmt;

use lint_layout::Name;

#[derive(Debug)]
pub(super) struct Finding {
    repeat: Name,
    first: Name,
}

impl Finding {
    pub(super) fn new(repeat: &Name, first: &Name) -> Finding {
        Finding { repeat: repeat.clone(), first: first.clone() }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "test {} differs from {} only in literals; make them rows of one laws::table!, or state the property with a law macro such as laws::roundtrip! or laws::idempotent!",
            self.repeat, self.first
        )
    }
}
