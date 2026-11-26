use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    Listed(Name),
    Spelled(Name),
    Mapped(Name),
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Listed(owner) => write!(formatter, "a const lists every variant of {owner} by hand"),
            Finding::Spelled(owner) => write!(formatter, "a const pairs variants of {owner} with their spellings"),
            Finding::Mapped(owner) => write!(formatter, "a fn maps each variant of {owner} to a literal"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "derive it: strum's VariantArray lists the variants; Display, IntoStaticStr or EnumString with #[strum(serialize = ...)] spell them",
        )
    }
}
