use std::ffi::OsStr;
use std::num::NonZeroU8;
use std::path::Path;

use crate::cli::ports::Invocation;

#[derive(Debug)]
pub(crate) struct CargoMutants;

impl CargoMutants {
    pub(crate) const MISSED: NonZeroU8 = NonZeroU8::MIN.saturating_add(1);
    pub(crate) const TIMEOUT: NonZeroU8 = Self::MISSED.saturating_add(1);

    const SUBCOMMAND: &'static str = "mutants";
    const RUNNER: &'static str =
        r#"--config=target.'cfg(target_os = "linux")'.runner = ["prlimit", "--data=6442450944", "--cpu=300", "--"]"#;
    const FIXED: [&'static str; 13] = [
        "--jobs",
        "2",
        "--colors",
        "never",
        "--no-shuffle",
        "--caught",
        "--unviable",
        "--build-timeout",
        "600",
        "--timeout",
        "120",
        "--cargo-test-arg",
        Self::RUNNER,
    ];
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
