use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding<'name> {
    ParameterBag { name: &'name Name },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::ParameterBag { name } = self;
        write!(formatter, "{name} only carries values into one other file")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::ParameterBag { .. } = self;
        Fix::new("give it the logic that consumes it")
    }
}
