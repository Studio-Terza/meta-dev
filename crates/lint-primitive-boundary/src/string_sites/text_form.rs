use std::fmt;

#[derive(Clone, Copy)]
pub enum TextForm {
    Owned,
    Borrowed,
    Unsized,
}

impl fmt::Display for TextForm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextForm::Owned => formatter.write_str("String"),
            TextForm::Borrowed => formatter.write_str("&str"),
            TextForm::Unsized => formatter.write_str("str"),
        }
    }
}
