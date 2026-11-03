use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Question {
    History,
    Affected,
    Widened,
}

impl From<Question> for OsString {
    fn from(question: Question) -> OsString {
        OsString::from(match question {
            Question::History => "history",
            Question::Affected => "affected",
            Question::Widened => "widened",
        })
    }
}
