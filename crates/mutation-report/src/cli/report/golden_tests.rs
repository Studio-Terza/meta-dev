mod boundary;
mod rust_files;

use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::io;
use std::path::Path;
use std::str::FromStr;

use boundary::GoldenTest;
use file_ports::Files;
use newtype::Newtype;
use rust_files::RustFiles;
use text_lines::LossyText;

use crate::cli::report::failure::Failure;
use crate::cli::report::test_name::TestName;

#[derive(Debug, Default)]
pub struct GoldenTests(BTreeSet<TestName>);

impl GoldenTests {
    const CRATES: &'static str = "crates";
    const TEST: &'static str = "#[test]";
    const ASYNC_TEST: &'static str = "#[tokio::test]";

    pub fn under(files: &impl Files, sources: &Path) -> Result<GoldenTests, Failure> {
        let found = RustFiles::below(files, &sources.join(Self::CRATES))?;
        found.iter().zip(files.read_many(&found)).try_fold(GoldenTests::default(), |tests, (file, bytes)| {
            let bytes =
                bytes.map_err(|failure| Failure::Io { path: file.clone(), source: io::Error::from(failure) })?;
            Ok(tests.merged(LossyText::from(bytes.as_ref()).as_ref().parse::<GoldenTests>().unwrap_or_default()))
        })
    }

    pub(crate) fn names(&self) -> &BTreeSet<TestName> {
        &self.0
    }

    fn merged(self, other: GoldenTests) -> GoldenTests {
        GoldenTests(self.0.into_iter().chain(other.0).collect())
    }
}

impl FromStr for GoldenTests {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<GoldenTests, Infallible> {
        let markers: BTreeMap<usize, usize> = [Self::TEST, Self::ASYNC_TEST]
            .into_iter()
            .flat_map(|marker| text.match_indices(marker).map(|(at, found)| (at, found.len())))
            .collect();
        let ends: Vec<usize> = markers.keys().skip(1).copied().chain([text.len()]).collect();
        Ok(GoldenTests(
            markers
                .iter()
                .zip(ends)
                .filter_map(|((start, length), end)| text.get(start + length..end))
                .filter_map(|chunk| chunk.parse::<GoldenTest>().ok())
                .map(GoldenTest::into_inner)
                .collect(),
        ))
    }
}
