use std::fmt::Display;
use std::process::ExitCode;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shown(String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Complaint(String);

#[derive(Debug)]
pub struct Answer {
    shown: Shown,
    complaint: Complaint,
    code: ExitCode,
}

impl Answer {
    #[must_use]
    pub fn new(code: ExitCode) -> Answer {
        Answer { shown: Shown::default(), complaint: Complaint::default(), code }
    }

    #[must_use]
    pub fn reported<Outcome, Failure: Display>(result: Result<Outcome, Failure>) -> Answer {
        result.map_or_else(
            |failure| Answer::complained(ExitCode::FAILURE, format_args!("{failure}\n")),
            |_outcome| Answer::new(ExitCode::SUCCESS),
        )
    }

    #[must_use]
    pub fn showing(self, text: impl Display) -> Answer {
        Answer { shown: Shown(format!("{}{text}", self.shown.0)), ..self }
    }

    #[must_use]
    pub fn complained(code: ExitCode, complaint: impl Display) -> Answer {
        Answer { shown: Shown::default(), complaint: Complaint(complaint.to_string()), code }
    }

    pub fn code(&self) -> ExitCode {
        self.code
    }
}

impl AsRef<Shown> for Answer {
    fn as_ref(&self) -> &Shown {
        &self.shown
    }
}

impl AsRef<Complaint> for Answer {
    fn as_ref(&self) -> &Complaint {
        &self.complaint
    }
}

impl AsRef<str> for Shown {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Complaint {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
