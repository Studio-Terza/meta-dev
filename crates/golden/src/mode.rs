use std::ffi::OsStr;

use console_ports::Environment;

use crate::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Check,
    Record,
}

impl Mode {
    pub const VARIABLE: &'static str = "GOLDEN_RECORD";
    pub const ON: &'static str = "1";
    const CI: &'static str = "CI";

    pub fn current(environment: &impl Environment) -> Result<Self, Failure> {
        Self::chosen(
            environment.variable(OsStr::new(Self::VARIABLE)).as_deref(),
            environment.variable(OsStr::new(Self::CI)).as_deref(),
        )
    }

    pub fn chosen(record: Option<&OsStr>, ci: Option<&OsStr>) -> Result<Self, Failure> {
        record.map_or(Ok(Self::Check), |value| Self::recording(value, ci))
    }

    fn recording(value: &OsStr, ci: Option<&OsStr>) -> Result<Self, Failure> {
        Some(value)
            .filter(|candidate| **candidate == *Self::ON)
            .ok_or_else(|| Failure::UnknownValue { value: value.to_owned() })
            .and_then(|_| ci.map_or(Ok(Self::Record), |_| Err(Failure::RecordInCi)))
    }
}
