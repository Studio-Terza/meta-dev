use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use file_ports::{Bytes, Text};

#[derive(Debug)]
pub struct Case {
    target: Text,
    role: Text,
    bytes: Bytes,
}

impl Case {
    const PENDING: [&'static str; 2] = ["rs", "role"];
    const HASH_DIGITS: usize = 16;

    #[must_use]
    pub const fn new(target: Text, role: Text, bytes: Bytes) -> Self {
        Self { target, role, bytes }
    }

    fn pending(directory: &Path, target: &Text) -> [PathBuf; Self::PENDING.len()] {
        Self::PENDING.map(|extension| directory.join(format!("{target}-pending.{extension}")))
    }

    fn kept(&self, directory: &Path) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        self.bytes.as_ref().hash(&mut hasher);
        let digits = Self::HASH_DIGITS;
        directory.join(format!("{}-{}-{:0digits$x}.rs", self.target, self.role, hasher.finish()))
    }
}

impl AsRef<[u8]> for Case {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Crashes;

impl Crashes {
    const SECONDS: &str = "FUZZ_SECONDS";
    const DIRECTORY: &str = "FUZZ_CRASHES";
    const DEFAULT_SECONDS: u64 = 5;

    fn directory() -> Option<PathBuf> {
        env::var_os(Self::DIRECTORY).map(PathBuf::from)
    }

    #[must_use]
    pub fn budget(self) -> Duration {
        Duration::from_secs(
            env::var(Self::SECONDS).ok().and_then(|text| text.parse().ok()).unwrap_or(Self::DEFAULT_SECONDS),
        )
    }

    pub fn note(self, case: &Case) -> io::Result<()> {
        Self::directory().map_or(Ok(()), |directory| {
            let [bytes, role] = Case::pending(&directory, &case.target);
            fs::create_dir_all(&directory)?;
            fs::write(bytes, case)?;
            fs::write(role, case.role.as_ref())
        })
    }

    pub fn clear(self, target: &Text) -> io::Result<()> {
        Self::directory()
            .map_or(Ok(()), |directory| Case::pending(&directory, target).iter().try_for_each(fs::remove_file))
    }

    #[must_use]
    pub fn keep(self, case: &Case) -> Option<PathBuf> {
        let directory = Self::directory()?;
        let file = case.kept(&directory);
        fs::create_dir_all(&directory).ok()?;
        fs::write(&file, case).ok()?;
        Some(file)
    }
}
