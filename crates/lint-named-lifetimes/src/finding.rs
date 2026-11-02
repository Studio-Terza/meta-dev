use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    ShortLifetime { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::ShortLifetime { name } = self;
        write!(formatter, "'{name}")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::ShortLifetime { .. } = self;
        Fix::new("name the lifetime with a word that says what it borrows from")
    }
}
