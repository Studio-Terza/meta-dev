use std::fmt;

use lint_layout::Name;

pub enum Finding {
    FreeText { context: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::FreeText { context } = self;
        write!(
            formatter,
            "free text in {context}; split it into typed parts, or use a newtype that validates (serde try_from, or a Deserialize that calls FromStr)"
        )
    }
}
