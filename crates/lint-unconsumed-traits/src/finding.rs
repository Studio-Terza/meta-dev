use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

use crate::declaration::Declaration;

pub(crate) struct Finding<'declaration>(&'declaration Declaration<'declaration>);

impl<'declaration> From<&'declaration Declaration<'declaration>> for Finding<'declaration> {
    fn from(declaration: &'declaration Declaration<'declaration>) -> Finding<'declaration> {
        Finding(declaration)
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trait {} has methods but nothing consumes it", AsRef::<Name>::as_ref(self.0))
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "delete the trait and keep its methods inherent, or give it a consumer: a bound, impl Trait, dyn Trait, or a provided method its callers use",
        )
    }
}
