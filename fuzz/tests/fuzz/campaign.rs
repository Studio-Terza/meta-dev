use std::collections::hash_map::DefaultHasher;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use lint_runner::Cli;
use proptest::prelude::Strategy;
use proptest::test_runner::{Config, TestCaseError, TestError, TestRunner};

use crate::crash::Crash;
use crate::input::Input;
use crate::pending::Pending;

const SECONDS: &str = "FUZZ_SECONDS";
const CRASHES: &str = "FUZZ_CRASHES";
const DEFAULT_SECONDS: u64 = 5;
const BATCH: u32 = 64;

pub struct Campaign;

impl Campaign {
    fn budget() -> Duration {
        Duration::from_secs(env::var(SECONDS).ok().and_then(|text| text.parse().ok()).unwrap_or(DEFAULT_SECONDS))
    }

    fn directory() -> Option<PathBuf> {
        env::var_os(CRASHES).map(PathBuf::from)
    }

    fn noted(target: &str, input: &Input) -> io::Result<()> {
        if let Some(pending) = Self::directory().map(|directory| Pending::of(&directory, target)) {
            pending.record(input)?;
        }
        Ok(())
    }

    fn cleared(target: &str) -> io::Result<()> {
        if let Some(pending) = Self::directory().map(|directory| Pending::of(&directory, target)) {
            pending.clear()?;
        }
        Ok(())
    }

    fn linted(target: &str, input: &Input) -> io::Result<()> {
        Self::noted(target, input)?;
        let root = tempfile::tempdir()?;
        input.written_into(root.path())?;
        let arguments = [OsString::from(root.path())];
        let _ = Cli::from_args(arguments.into_iter()).report_to(&mut Vec::new(), &mut Vec::new());
        Ok(())
    }

    fn kept(target: &str, input: &Input) -> Option<PathBuf> {
        let mut hasher = DefaultHasher::new();
        input.bytes().hash(&mut hasher);
        let file = Self::directory()?.join(format!("{target}-{}-{:016x}.rs", input.name(), hasher.finish()));
        fs::create_dir_all(file.parent()?).ok()?;
        fs::write(&file, input.bytes()).ok()?;
        Some(file)
    }

    pub fn run(target: &str, strategy: impl Strategy<Value = Input>) -> Result<(), Crash> {
        let deadline = Instant::now() + Self::budget();
        while Instant::now() < deadline {
            let config = Config { cases: BATCH, failure_persistence: None, ..Config::default() };
            TestRunner::new(config)
                .run(&strategy, |input| {
                    Self::linted(target, &input).map_err(|failure| TestCaseError::reject(failure.to_string()))
                })
                .map_err(|failure| match failure {
                    TestError::Fail(reason, input) => {
                        Crash::Panicked { kept: Self::kept(target, &input), reason: reason.message().to_owned(), input }
                    }
                    TestError::Abort(reason) => Crash::Aborted { reason: reason.message().to_owned() },
                })?;
        }
        Self::cleared(target).map_err(|failure| Crash::Aborted { reason: failure.to_string() })
    }
}
