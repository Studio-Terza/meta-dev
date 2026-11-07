use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum Document {
    Limits,
    Measured,
}

impl fmt::Display for Document {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limits => formatter.write_str("limits"),
            Self::Measured => formatter.write_str("measured"),
        }
    }
}
