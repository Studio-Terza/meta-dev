use std::fmt;
use std::path::Path;

use design::{Lint, Verdict};
use file_ports::{Disk, Files};
use test_failure::{Failed, Golden};
use timer_ports::Monotonic;

const FIXTURES: &str = "tests/.fixtures";
const GOLDENS: &str = "tests/goldens";

#[derive(Debug)]
struct Linted {
    verdict: Verdict,
    out: String,
    err: String,
}

#[test]
fn every_lint_fixture_prints_its_golden_findings() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    Disk.entries(&manifest.join(FIXTURES))?.iter().try_for_each(|fixture| {
        let name = fixture.file_name().map_or_default(|named| named.to_string_lossy().into_owned());
        Golden::at(manifest, &format!("{GOLDENS}/{name}.txt")).checked(&Linted::of(fixture)?.to_string())
    })
}

impl Linted {
    fn of(fixture: &Path) -> Result<Linted, Failed> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let verdict = Lint::from_args([]).verdict(&Disk, fixture, Monotonic::default(), &mut out, &mut err);
        Ok(Linted { verdict, out: String::from_utf8(out)?, err: String::from_utf8(err)? })
    }
}

impl fmt::Display for Linted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}\n--- out\n{}--- err\n{}", self.verdict, self.out, self.err)
    }
}
