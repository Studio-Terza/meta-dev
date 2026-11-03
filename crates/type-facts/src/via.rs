use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Via {
    Field,
    Variant,
    Bound,
}

impl fmt::Display for Via {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let said = match self {
            Via::Field => "field",
            Via::Variant => "variant",
            Via::Bound => "bound",
        };
        formatter.write_str(said)
    }
}
