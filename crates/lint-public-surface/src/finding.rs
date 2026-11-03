use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding<'export> {
    UnnamedExport { name: &'export Name },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::UnnamedExport { name } = self;
        write!(formatter, "{name} is exported but no other crate names it")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::UnnamedExport { .. } = self;
        Fix::new("export it when a user appears")
    }
}
