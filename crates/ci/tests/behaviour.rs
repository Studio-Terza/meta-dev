use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use tempfile::TempDir;
use test_failure::{Cases, Failed};

const CASES: &str = "tests/cases.toml";

#[derive(Debug, Deserialize)]
struct Fixture {
    case: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    arguments: Vec<String>,
    #[serde(default)]
    files: BTreeMap<String, String>,
    code: i32,
    #[serde(default)]
    out: String,
    #[serde(default)]
    err: String,
    #[serde(default)]
    written: BTreeMap<String, String>,
}

mod ports {
    use std::io;
    use std::path::Path;
    use std::process::{Command, Output};

    #[derive(Debug)]
    pub(crate) struct Binary;

    impl Binary {
        pub(crate) fn ran(directory: &Path, arguments: &[String]) -> io::Result<Output> {
            Command::new(env!("CARGO_BIN_EXE_ci")).args(arguments).current_dir(directory).output()
        }
    }
}

#[test]
fn every_case_exits_prints_and_writes_as_expected() -> Result<(), Failed> {
    let cases: Fixture = Cases::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(CASES))?;
    let drifted: Vec<Option<String>> = cases.case.iter().map(Case::drifted).collect::<Result<_, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<String>>(), Vec::<String>::new());
    Ok(())
}

impl Case {
    fn drifted(&self) -> Result<Option<String>, Failed> {
        let directory = TempDir::new()?;
        self.files.iter().try_for_each(|(path, text)| fs::write(directory.path().join(path), text))?;
        let ran = ports::Binary::ran(directory.path(), &self.arguments)?;
        let [printed, complained] = [ran.stdout, ran.stderr].map(String::from_utf8);
        let [out, err] = [printed?, complained?];
        let written: BTreeMap<String, String> = self
            .written
            .keys()
            .map(|path| Ok((path.clone(), fs::read_to_string(directory.path().join(path))?)))
            .collect::<Result<_, Failed>>()?;
        let held = ran.status.code() == Some(self.code)
            && out.get(..self.out.len()) == Some(self.out.as_str())
            && err.get(..self.err.len()) == Some(self.err.as_str())
            && written == self.written;
        Ok((!held).then(|| format!("{:?}: exit {:?}\n{out}{err}{written:?}", self.arguments, ran.status.code())))
    }
}
