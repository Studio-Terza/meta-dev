use std::env;
use std::ffi::OsStr;

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

    pub fn current() -> Result<Self, Failure> {
        Self::chosen(env::var_os(Self::VARIABLE).as_deref(), env::var_os(Self::CI).as_deref())
    }

    pub fn chosen(record: Option<&OsStr>, ci: Option<&OsStr>) -> Result<Self, Failure> {
        match (record, ci) {
            (None, _) => Ok(Self::Check),
            (Some(value), _) if value != Self::ON => Err(Failure::UnknownValue { value: value.to_owned() }),
            (Some(_), None) => Ok(Self::Record),
            (Some(_), Some(_)) => Err(Failure::RecordInCi),
        }
    }
}
