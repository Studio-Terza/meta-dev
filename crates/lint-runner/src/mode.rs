use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Gate,
    Report,
}

impl Mode {
    pub(crate) const REPORT_FLAG: &'static str = "--report";

    pub(crate) fn of_arguments(arguments: &[PathBuf]) -> Mode {
        arguments.iter().find(|argument| argument.as_os_str() == Self::REPORT_FLAG).map_or(Mode::Gate, |_| Mode::Report)
    }
}
