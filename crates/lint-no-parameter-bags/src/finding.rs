use std::fmt;

use lint_layout::Name;

pub enum Finding {
    ParameterBag { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::ParameterBag { name } = self;
        write!(formatter, "{name} only carries values into one other file; give it the logic that consumes it")
    }
}
