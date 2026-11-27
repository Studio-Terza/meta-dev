use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt, fs};

use invariants::Ignoring;
use serde::de::DeserializeOwned;
use test_failure::Failed;

invariants::ignored! {
    enum Ignored {
        Outside => "a story found outside the manifest directory is read from where it was found",
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Story {
    file: &'static str,
    manifest: &'static str,
    name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaseName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Golden {
    path: PathBuf,
    text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    case: CaseName,
    printed: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(transparent)]
pub struct Spelled(String);

impl fmt::Debug for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

#[derive(Debug)]
pub struct Debugged<Shown>(pub Shown);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parsed<Read>(Read);

impl<Read> Parsed<Read> {
    pub fn value(&self) -> &Read {
        &self.0
    }
}

impl<'de, Read: FromStr<Err: fmt::Display>> serde::Deserialize<'de> for Parsed<Read> {
    fn deserialize<Reader: serde::Deserializer<'de>>(reader: Reader) -> Result<Parsed<Read>, Reader::Error> {
        let text = String::deserialize(reader)?;
        text.parse().map(Parsed).map_err(serde::de::Error::custom)
    }
}

impl<Read: fmt::Debug> fmt::Debug for Parsed<Read> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl Spelled {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Spelled {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<Shown: fmt::Debug> fmt::Display for Debugged<Shown> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{:#?}", self.0)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Entry {
    directory: Option<PathBuf>,
    golden: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
enum Unread {
    #[error("no story file {story} under {manifest}")]
    Unanchored { story: &'static str, manifest: &'static str },
    #[error("case {0} has no golden file beside it")]
    Unpaired(CaseName),
    #[error("case {0} has no row in the story's table")]
    Unrowed(CaseName),
}

impl CaseName {
    pub(crate) fn unrowed(&self) -> Failed {
        Failed::from(Unread::Unrowed(self.clone()))
    }
}

impl Story {
    const TABLE: &'static str = "toml";
    const RECORD: &'static str = "FIXTURE_RECORD";

    #[must_use]
    pub const fn anchored(file: &'static str, manifest: &'static str, name: &'static str) -> Story {
        Story { file, manifest, name }
    }

    fn path(&self) -> Result<PathBuf, Failed> {
        let manifest = Path::new(self.manifest);
        let found = manifest.ancestors().map(|above| above.join(self.file)).find(|path| path.is_file());
        found
            .map(|path| {
                path.strip_prefix(manifest).ignoring(Ignored::Outside).map_or_else(|| path.clone(), Path::to_path_buf)
            })
            .ok_or_else(|| Failed::from(Unread::Unanchored { story: self.file, manifest: self.manifest }))
    }

    pub(crate) fn rows<Row: DeserializeOwned>(&self) -> Result<BTreeMap<CaseName, Row>, Failed> {
        let table = self.path()?.with_extension(Story::TABLE);
        let text = fs::read_to_string(&table).or_else(Story::absent)?;
        let mut parsed: toml::Table = toml::from_str(&text)?;
        let rows: BTreeMap<String, Row> =
            parsed.remove(self.name).map_or(Ok(BTreeMap::new()), toml::Value::try_into)?;
        Ok(rows.into_iter().map(|(name, row)| (CaseName(name), row)).collect())
    }

    pub(crate) fn entries(&self) -> Result<BTreeMap<CaseName, Entry>, Failed> {
        let directory = self.path()?.with_extension(OsString::new());
        let listed = fs::read_dir(&directory)?
            .map(|entry| entry.map(|found| found.path()))
            .collect::<io::Result<Vec<PathBuf>>>()?;
        Ok(listed.into_iter().fold(BTreeMap::new(), |mut entries, path| {
            let named = if path.is_dir() { path.file_name() } else { path.file_stem() };
            let name = CaseName(named.map_or_else(String::new, |stem| stem.to_string_lossy().into_owned()));
            let entry: &mut Entry = entries.entry(name).or_default();
            entry.placed(path);
            entries
        }))
    }

    fn absent(failure: io::Error) -> Result<String, io::Error> {
        (failure.kind() == ErrorKind::NotFound).then(String::new).ok_or(failure)
    }
}

impl Entry {
    fn placed(&mut self, path: PathBuf) {
        let directory = path.is_dir().then(|| path.clone());
        self.golden = self.golden.take().or(directory.is_none().then_some(path));
        self.directory = self.directory.take().or(directory);
    }

    pub(crate) fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    pub(crate) fn golden(&self, name: &CaseName) -> Result<Golden, Failed> {
        let path = self.golden.clone().ok_or_else(|| Unread::Unpaired(name.clone()))?;
        let text = fs::read_to_string(&path)?;
        Ok(Golden { path, text })
    }
}

impl Golden {
    pub(crate) fn compared(&self, case: &CaseName, shown: impl fmt::Display) -> Result<Option<Drift>, Failed> {
        let printed = shown.to_string();
        let recording = env::var_os(Story::RECORD).map(|_set| ());
        recording.map_or(Ok(()), |()| fs::write(&self.path, &printed))?;
        let drifted = recording.xor(Some(())).filter(|()| printed != self.text);
        Ok(drifted.map(|()| Drift { case: case.clone(), printed }))
    }
}

impl fmt::Display for CaseName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
