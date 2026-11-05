use std::ffi::OsStr;
use std::num::NonZeroI32;
use std::path::Path;

use crate::cli::ports::Invocation;

#[derive(Debug)]
pub struct CargoMutants;

impl CargoMutants {
    pub(crate) const MISSED: NonZeroI32 = NonZeroI32::new(2).unwrap();
    pub(crate) const TIMEOUT: NonZeroI32 = NonZeroI32::new(3).unwrap();

    const SUBCOMMAND: &'static str = "mutants";
    const FIXED: [&'static str; 4] = ["--jobs", "2", "--colors", "never"];
    const OUTPUT: &'static str = "--output";

    pub(crate) fn invocation(output: &Path, selection: &[&OsStr], passed_on: &[&OsStr]) -> Invocation {
        Invocation::new(
            "cargo",
            [OsStr::new(Self::SUBCOMMAND)]
                .into_iter()
                .chain(selection.iter().copied())
                .chain(Self::FIXED.iter().map(OsStr::new))
                .chain([OsStr::new(Self::OUTPUT), output.as_os_str()])
                .chain(passed_on.iter().copied()),
        )
    }
}
