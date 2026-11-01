use std::fmt;

use lint_core::Most;
use lint_core::TypeName;
use lint_layout::Name;

pub enum Finding {
    TooManyPrivateFns { type_name: TypeName, names: Vec<Name>, most: Most },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::TooManyPrivateFns { type_name, names, most } = self;
        write!(
            formatter,
            "{type_name} has {} private fns, at most {most} allowed: {}",
            names.len(),
            names.iter().map(ToString::to_string).collect::<Vec<String>>().join(", ")
        )
    }
}
