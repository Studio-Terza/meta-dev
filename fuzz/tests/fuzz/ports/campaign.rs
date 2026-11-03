use std::ffi::OsString;
use std::path::Path;
use std::time::Instant;

use file_ports::{Bytes, Memory, Text};
use fuzz::ports::{Case, Crashes};
use lint_runner::Cli;
use proptest::prelude::Strategy;
use proptest::test_runner::{Config, TestCaseError, TestError, TestRunner};

use crate::crash::Crash;
use crate::input::Input;

const ROOT: &str = "/fuzzed";
const BATCH: u32 = 64;

pub struct Campaign;

impl Campaign {
    fn case(target: &str, input: &Input) -> Case {
        Case::new(Text::new(target.to_owned()), Text::new(input.name().to_owned()), Bytes::new(input.bytes().to_vec()))
    }

    fn linted(crashes: Crashes, target: &str, input: &Input) -> Result<(), TestCaseError> {
        crashes.note(&Self::case(target, input)).map_err(|failure| TestCaseError::reject(failure.to_string()))?;
        let root = Path::new(ROOT);
        let files: Memory = [
            (root.join("Cargo.toml"), Bytes::new(input.manifest().to_vec())),
            (root.join(input.file()), Bytes::new(input.source().to_vec())),
        ]
        .into_iter()
        .collect();
        let arguments = [OsString::from(root)];
        let _ = Cli::from_args(arguments.into_iter()).report_to(&files, &mut Vec::new(), &mut Vec::new());
        Ok(())
    }

    pub fn run(target: &str, strategy: impl Strategy<Value = Input>) -> Result<(), Crash> {
        let crashes = Crashes;
        let deadline = Instant::now() + crashes.budget();
        while Instant::now() < deadline {
            let config = Config { cases: BATCH, failure_persistence: None, ..Config::default() };
            TestRunner::new(config).run(&strategy, |input| Self::linted(crashes, target, &input)).map_err(
                |failure| match failure {
                    TestError::Fail(reason, input) => Crash::Panicked {
                        kept: crashes.keep(&Self::case(target, &input)),
                        reason: reason.message().to_owned(),
                        input,
                    },
                    TestError::Abort(reason) => Crash::Aborted { reason: reason.message().to_owned() },
                },
            )?;
        }
        crashes.clear(&Text::new(target.to_owned())).map_err(|failure| Crash::Aborted { reason: failure.to_string() })
    }
}
