use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use file_ports::Disk;
use lint::{Cli, Verdict};
use test_failure::{Failed, Golden};
use timer_ports::Monotonic;

const RUNS: [&[&str]; 4] =
    [&["skipped"], &["workspace"], &["/nowhere", "bad_manifest/notes"], &["workspace/app", "workspace/store-ports"]];

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
fn every_run_reads_the_disk_as_its_arguments_say_and_ends_in_its_verdict() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = Fixtures(manifest.join("tests/.fixtures"));
    let shown = RUNS.iter().map(|arguments| Ok(fixtures.ran(arguments)?.shown(arguments)));
    Golden::at(manifest, "tests/runs.txt").checked(&shown.collect::<Result<String, Failed>>()?)?;
    let [timed, failed] = [fixtures.ran(&["--timings", "skipped"])?.out, fixtures.ran(&["--timings", "/nowhere"])?.out];
    assert!(timed.contains("run:") && !timed.contains(" 0  run:") && failed.contains("find:"));
    let mut err = Vec::new();
    let unwritten =
        Cli::from_args([]).verdict(&Disk, &fixtures.0.join("skipped"), Monotonic::default(), &mut Unwritable, &mut err);
    assert_eq!(
        (unwritten, String::from_utf8(err)?),
        (Verdict::Refused, "cannot write the output: refused\n".to_owned())
    );
    assert_eq!(["/nowhere", "."].map(ports::Binary::exit), [Some(1_i32), Some(0_i32)]);
    Ok(())
}

impl Fixtures {
    fn ran(&self, arguments: &[&str]) -> Result<Ran, Failed> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let given = arguments.iter().map(OsString::from);
        let verdict = Cli::from_args(given).verdict(&Disk, &self.0, Monotonic::default(), &mut out, &mut err);
        Ok(Ran { verdict, out: String::from_utf8(out)?, err: String::from_utf8(err)? })
    }
}

impl Ran {
    fn shown(&self, arguments: &[&str]) -> String {
        format!("$ lint {}\n{:?}\n--- out\n{}--- err\n{}", arguments.join(" "), self.verdict, self.out, self.err)
    }
}

mod ports {
    use std::process::Command;

    #[derive(Debug)]
    pub(crate) struct Binary;

    impl Binary {
        pub(crate) fn exit(root: &str) -> Option<i32> {
            Command::new(env!("CARGO_BIN_EXE_lint")).arg(root).output().into_iter().find_map(|ran| ran.status.code())
        }
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
