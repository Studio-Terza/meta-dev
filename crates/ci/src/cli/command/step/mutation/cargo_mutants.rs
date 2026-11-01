use std::ffi::OsStr;
use std::num::NonZeroU8;
use std::path::Path;

use crate::cli::ports::Invocation;

#[derive(Debug)]
pub struct CargoMutants;

impl CargoMutants {
    pub(crate) const MISSED: NonZeroU8 = NonZeroU8::MIN.saturating_add(1);
    pub(crate) const TIMEOUT: NonZeroU8 = Self::MISSED.saturating_add(1);

    const SUBCOMMAND: &'static str = "mutants";
    const FIXED: [&'static str; 4] = ["--jobs", "2", "--colors", "never"];
    const OUTPUT: &'static str = "--output";

    pub(crate) fn invocation(output: &Path, selection: &[&OsStr], passed_on: &[&str]) -> Invocation {
        Invocation::new(
            "cargo",
            [OsStr::new(Self::SUBCOMMAND)]
                .into_iter()
                .chain(selection.iter().copied())
                .chain(Self::FIXED.iter().map(OsStr::new))
                .chain([OsStr::new(Self::OUTPUT), output.as_os_str()])
                .chain(passed_on.iter().map(OsStr::new)),
        )
    }
}
