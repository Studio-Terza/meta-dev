use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Question {
    History,
    Affected,
    Widened,
}

impl From<Question> for OsString {
    fn from(question: Question) -> Self {
        match question {
            Question::History => Self::from("history"),
            Question::Affected => Self::from("affected"),
            Question::Widened => Self::from("widened"),
        }
    }
}
