use std::fmt;

use lint_layout::Name;

use crate::declaration::Declaration;

pub(crate) struct Finding<'declaration>(&'declaration Declaration);

impl<'declaration> From<&'declaration Declaration> for Finding<'declaration> {
    fn from(declaration: &'declaration Declaration) -> Finding<'declaration> {
        Finding(declaration)
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "trait {} has no generic consumer; use a standard trait such as From, AsRef or Display, else write the generic over an existing trait, else a derive",
            AsRef::<Name>::as_ref(self.0)
        )
    }
}
