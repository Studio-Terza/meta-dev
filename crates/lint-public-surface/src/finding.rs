use std::fmt;

use lint_layout::Name;

pub enum Finding {
    UnnamedExport { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::UnnamedExport { name } = self;
        write!(formatter, "{name} is exported but no other crate names it; export it when a user appears")
    }
}
