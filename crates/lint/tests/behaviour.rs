use std::path::PathBuf;

use file_doubles::Memory;
use fixture::{Cases, Failed, Fixture};
use lint::{Cli, Verdict};
use serde::Deserialize;
use timer_ports::Stopped;

#[derive(Fixture, Deserialize, Debug)]
struct Lint {
    arguments: Vec<PathBuf>,
    verdict: Verdict,
    files: Memory,
    #[serde(default)]
    out: Out,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Out {
    #[default]
    Kept,
    Full,
}

#[test]
fn every_case_prints_its_golden_output_and_reaches_its_verdict() -> Result<(), Failed> {
    let drifted = Cases::<Lint>::drifted(|case| {
        let name = case.name().clone();
        let Lint { arguments, verdict: expected, files, out } = case.into_row();
        let cli = Cli::from_args(arguments.into_iter().map(PathBuf::into_os_string));
        let mut kept = Vec::new();
        let mut err = Vec::new();
        let verdict = match out {
            Out::Kept => cli.verdict(files, Stopped::default(), &mut kept, &mut err),
            Out::Full => {
                let mut full: &mut [u8] = &mut [];
                cli.verdict(files, Stopped::default(), &mut full, &mut err)
            }
        };
        assert_eq!((&name, verdict), (&name, expected));
        Ok(String::from_utf8([kept, err].concat())?)
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
