use std::fmt;

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
