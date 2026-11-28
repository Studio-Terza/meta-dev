use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt, fs};

use serde::de::DeserializeOwned;
use test_failure::Failed;

#[derive(Debug, thiserror::Error)]
#[error("{call} answered {implemented} where the reference answered {referenced}")]
pub(crate) struct Disagreed {
    call: String,
    implemented: String,
    referenced: String,
}

impl Disagreed {
    pub(crate) fn between(
        call: &impl fmt::Debug,
        implemented: &impl fmt::Debug,
        referenced: &impl fmt::Debug,
    ) -> Disagreed {
        Disagreed {
            call: format!("{call:?}"),
            implemented: format!("{implemented:?}"),
            referenced: format!("{referenced:?}"),
        }
    }
}

pub trait Fixture: DeserializeOwned {
    const FILE: &'static str;
    const MANIFEST: &'static str;
    const NAME: &'static str;
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

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(transparent)]
pub struct Spelled(String);

impl fmt::Debug for Spelled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

#[derive(Debug)]
pub struct Debugged<Shown>(pub Shown);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl Spelled {
    #[must_use]
    pub fn as_str(&self) -> &str {
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
    #[error("case {} has no golden file beside it", .0.0)]
    Unpaired(CaseName),
}

impl Story {
    const TABLE: &'static str = "toml";
    const RECORD: &'static str = "FIXTURE_RECORD";

    #[must_use]
    pub(crate) const fn of<Row: Fixture>() -> Story {
        Story { file: Row::FILE, manifest: Row::MANIFEST, name: Row::NAME }
    }

    fn path(&self) -> Result<PathBuf, Failed> {
        let manifest = Path::new(self.manifest);
        let found = manifest.ancestors().map(|above| above.join(self.file)).find(|path| path.is_file());
        found.ok_or_else(|| Failed::from(Unread::Unanchored { story: self.file, manifest: self.manifest }))
    }

    pub(crate) fn rows<Row: DeserializeOwned>(&self) -> Result<BTreeMap<CaseName, Row>, Failed> {
        let table = self.path()?.with_extension(Story::TABLE);
        let text = fs::read_to_string(&table)?;
        let mut parsed: toml::Table = toml::from_str(&text)?;
        let rows: BTreeMap<String, Row> = match parsed.remove(self.name) {
            Some(toml::Value::Array(listed)) => listed
                .into_iter()
                .enumerate()
                .map(|(at, row)| Ok((at.to_string(), row.try_into()?)))
                .collect::<Result<_, toml::de::Error>>()?,
            Some(keyed) => keyed.try_into()?,
            None => BTreeMap::new(),
        };
        Ok(rows.into_iter().map(|(name, row)| (CaseName(name), row)).collect())
    }

    pub(crate) fn cases(&self) -> Result<PathBuf, Failed> {
        Ok(self.path()?.with_extension(OsString::new()))
    }

    pub(crate) fn entries(&self) -> Result<BTreeMap<CaseName, Entry>, Failed> {
        let directory = self.cases()?;
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
