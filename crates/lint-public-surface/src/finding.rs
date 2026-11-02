use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    UnnamedExport { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::UnnamedExport { name } = self;
        write!(formatter, "{name} is exported but no other crate names it")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::UnnamedExport { .. } = self;
        Fix::new("export it when a user appears")
    }
}
