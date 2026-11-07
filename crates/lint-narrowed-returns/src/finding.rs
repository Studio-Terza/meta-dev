use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

use crate::narrowed::Narrowed;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Finding<'model> {
    name: &'model Name,
    narrowed: Narrowed,
}

impl<'model> Finding<'model> {
    pub(crate) fn new(name: &'model Name, narrowed: Narrowed) -> Self {
        Finding { name, narrowed }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "return what the check found (a parsed type, an enum to match on, a proof type) or a receipt of the effect (Written, Removed, Released, Spawned)",
        )
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} -> {}", self.name, self.narrowed.shown())
    }
}
