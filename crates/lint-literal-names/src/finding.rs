use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding<'name> {
    VagueName { name: &'name Name },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::VagueName { name } = self;
        write!(formatter, "fn {name} names nothing")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::VagueName { .. } = self;
        Fix::new("say what it does")
    }
}
