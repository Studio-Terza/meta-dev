use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Question {
    History,
    Affected,
    Widened,
}

impl From<Question> for OsString {
    fn from(question: Question) -> OsString {
        match question {
            Question::History => OsString::from("history"),
            Question::Affected => OsString::from("affected"),
            Question::Widened => OsString::from("widened"),
        }
    }
}
