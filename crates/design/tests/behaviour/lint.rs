use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use design::{Lint, Verdict};
use file_ports::Disk;
use test_failure::{Failed, Golden};
use timer_ports::Monotonic;

const RUNS: [&[&str]; 3] =
    [&["workspace"], &["/nowhere", "bad_manifest/notes"], &["workspace/app", "workspace/store-ports"]];

#[derive(Debug)]
struct Fixtures(PathBuf);

#[derive(Debug)]
struct Ran {
    verdict: Verdict,
    out: String,
    err: String,
}

#[derive(Debug)]
struct Unwritable;

#[test]
fn every_lint_run_reads_the_disk_as_its_arguments_say_and_ends_in_its_verdict() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = Fixtures(manifest.join("tests/.lint"));
    let shown = RUNS.iter().map(|arguments| Ok(fixtures.ran(arguments)?.shown(arguments)));
    Golden::at(manifest, "tests/behaviour/runs.txt").checked(&shown.collect::<Result<String, Failed>>()?)?;
    let [timed, failed] =
        [fixtures.ran(&["--timings", "workspace/store-ports"])?.out, fixtures.ran(&["--timings", "/nowhere"])?.out];
    assert!(timed.contains("run:") && !timed.contains(" 0  run:") && failed.contains("find:"));
    let mut err = Vec::new();
    let clean = fixtures.0.join("workspace/store-ports");
    let unwritten = Lint::from_args([]).verdict(&Disk, &clean, Monotonic::default(), &mut Unwritable, &mut err);
    assert_eq!(
        (unwritten, String::from_utf8(err)?),
        (Verdict::Refused, "cannot write the output: refused\n".to_owned())
    );
    Ok(())
}

impl Fixtures {
    fn ran(&self, arguments: &[&str]) -> Result<Ran, Failed> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let asked = ["lint"].iter().chain(arguments).map(OsString::from).collect::<Vec<OsString>>();
        let lint = Lint::asked(&asked).ok_or("a lint run is asked for by name")?;
        let verdict = lint.verdict(&Disk, &self.0, Monotonic::default(), &mut out, &mut err);
        Ok(Ran { verdict, out: String::from_utf8(out)?, err: String::from_utf8(err)? })
    }
}

impl Ran {
    fn shown(&self, arguments: &[&str]) -> String {
        format!("$ design lint {}\n{:?}\n--- out\n{}--- err\n{}", arguments.join(" "), self.verdict, self.out, self.err)
    }
}

impl Write for Unwritable {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("refused"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
