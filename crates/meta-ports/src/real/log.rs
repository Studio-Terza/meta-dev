use crate::{Log, PortError};
use meta_core::Record;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

/// meta-dev's data directory: the log, and the cache of `self/` files last read from main.
pub(super) fn home() -> Result<PathBuf, PortError> {
    data_dir().map(|(home, _)| home)
}

/// The data directory, and whether it is the default one, `$HOME/.local/share/meta-dev`, which
/// holds the lead's real log. XDG_DATA_HOME naming any other place makes it a scratch directory.
fn data_dir() -> Result<(PathBuf, bool), PortError> {
    data_dir_from(std::env::var_os("XDG_DATA_HOME").map(PathBuf::from), std::env::var_os("HOME").map(PathBuf::from))
}

fn data_dir_from(chosen: Option<PathBuf>, home: Option<PathBuf>) -> Result<(PathBuf, bool), PortError> {
    let default = home.map(|home| home.join(".local").join("share").join(DIR));
    let chosen = chosen.map(|chosen| chosen.join(DIR));
    let real = match (&chosen, &default) {
        (None, _) => true,
        (Some(chosen), Some(default)) => same_log(chosen, default),
        (Some(_), None) => false,
    };
    chosen
        .or(default)
        .map(|data| (data, real))
        .ok_or_else(|| PortError::new("neither XDG_DATA_HOME nor HOME is set; meta-dev has no data directory"))
}

/// meta-dev's directory under the data home, and the log in it.
const DIR: &str = "meta-dev";
const LOG: &str = "log.jsonl";

/// Two meta-dev directories that write one log: the same directory, or the same log file, by any
/// name, through symlinks too.
fn same_log(one: &Path, other: &Path) -> bool {
    let same = |one: &Path, other: &Path| resolved(one) == resolved(other);
    same(one, other) || same(&one.join(LOG), &other.join(LOG))
}

/// The path with symlinks resolved as far as it exists: a part not yet created keeps its name
/// under its resolved parent.
fn resolved(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => resolved(parent).join(name),
        _ => path.to_path_buf(),
    })
}

#[derive(Debug)]
pub struct JsonlLog {
    path: Result<PathBuf, PortError>,
    patience: Duration,
    real: bool,
}

impl JsonlLog {
    pub const PATIENCE: Duration = Duration::from_secs(3);
    const RETRY: Duration = Duration::from_millis(20);

    /// The log in the data directory. With no data directory it is taken as the real one: fail closed.
    pub fn in_data_dir() -> Self {
        let (path, real) = match data_dir() {
            Ok((home, real)) => (Ok(home.join(LOG)), real),
            Err(error) => (Err(error), true),
        };
        Self { path, patience: Self::PATIENCE, real }
    }

    /// A scratch log at `path`.
    pub fn at(path: PathBuf) -> Self {
        Self { path: Ok(path), patience: Self::PATIENCE, real: false }
    }

    fn locked(&self) -> Result<(File, &Path), PortError> {
        let path = self.path.as_ref().map_err(Clone::clone)?;
        path.parent().map_or(Ok(()), fs::create_dir_all).map_err(|error| Self::failed(path, error))?;
        let file = OpenOptions::new().create(true).read(true).append(true).open(path);
        let file = file.map_err(|error| Self::failed(path, error))?;
        let deadline = Instant::now() + self.patience;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok((file, path)),
                Err(TryLockError::WouldBlock) if Instant::now() < deadline => thread::sleep(Self::RETRY),
                Err(TryLockError::WouldBlock) => {
                    let waited = self.patience.as_millis();
                    return Err(PortError::new(format!("{} stayed locked for {waited} ms", path.display())));
                }
                Err(TryLockError::Error(error)) => return Err(Self::failed(path, error)),
            }
        }
    }

    fn failed(path: &Path, error: std::io::Error) -> PortError {
        PortError::new(format!("{}: {error}", path.display()))
    }

    fn write(file: &mut File, path: &Path, records: &[Record]) -> Result<(), PortError> {
        let lines = records
            .iter()
            .map(|record| serde_json::to_string(record).map(|line| line + "\n"))
            .collect::<Result<String, _>>()
            .map_err(|error| PortError::new(error.to_string()))?;
        file.write_all(lines.as_bytes()).map_err(|error| Self::failed(path, error))
    }

    fn contents(file: &mut File, path: &Path) -> Result<Vec<Record>, PortError> {
        let mut text = String::new();
        file.read_to_string(&mut text).map_err(|error| Self::failed(path, error))?;
        super::jsonl::records(path, &text)
    }
}

impl Log for JsonlLog {
    fn place(&self) -> String {
        self.path.as_ref().map_or_else(|_| "the meta-dev log".to_owned(), |path| path.display().to_string())
    }

    fn is_real(&self) -> bool {
        self.real
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        let (mut file, path) = self.locked()?;
        Self::contents(&mut file, path)
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        let (mut file, path) = self.locked()?;
        let (records, decided) = decide(Self::contents(&mut file, path));
        Self::write(&mut file, path, &records).map(|()| decided)
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonlLog, data_dir_from};
    use crate::Log;
    use std::fs::{File, TryLockError};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn a_transaction_holds_the_log_exclusively_from_read_to_append() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("meta-dev-lock-{nanos}.jsonl"));
        let mut log = JsonlLog::at(path.clone());
        let blocked = log
            .transact(|records| {
                let other = File::open(&path).unwrap();
                (Vec::new(), (records.unwrap().len(), matches!(other.try_lock(), Err(TryLockError::WouldBlock))))
            })
            .unwrap();
        let free = JsonlLog::at(path.clone()).transact(|records| (records.unwrap(), ())).is_ok();
        std::fs::remove_file(&path).unwrap();
        assert_eq!((blocked, free), ((0, true), true));
    }

    #[test]
    fn only_the_default_data_dir_holds_the_real_log_and_any_other_xdg_data_home_is_scratch() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let home = std::env::temp_dir().join(format!("meta-dev-home-{nanos}"));
        let share = home.join(".local").join("share");
        std::fs::create_dir_all(&share).unwrap();
        let alias = home.join("alias");
        std::os::unix::fs::symlink(&share, &alias).unwrap();
        let real = |chosen: Option<&std::path::Path>| {
            data_dir_from(chosen.map(std::path::Path::to_path_buf), Some(home.clone())).unwrap().1
        };
        let decided = [real(None), real(Some(&share)), real(Some(&alias)), real(Some(&home.join("scratch")))];
        let homeless = data_dir_from(Some(home.join("scratch")), None).unwrap().1;
        std::fs::remove_dir_all(&home).unwrap();
        assert_eq!((decided, homeless), ([true, true, true, false], false));
        assert_eq!(data_dir_from(None, None).map(|(_, real)| real).ok(), None);
        assert!(!JsonlLog::at(home).is_real());
    }

    #[test]
    fn a_scratch_data_home_whose_meta_dev_dir_or_log_links_to_the_real_one_holds_the_real_log() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let home = std::env::temp_dir().join(format!("meta-dev-linked-{nanos}"));
        let real_dir = home.join(".local").join("share").join("meta-dev");
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(real_dir.join("log.jsonl"), "").unwrap();
        let (linked_dir, linked_log) = (home.join("linked-dir"), home.join("linked-log"));
        std::fs::create_dir_all(&linked_dir).unwrap();
        std::os::unix::fs::symlink(&real_dir, linked_dir.join("meta-dev")).unwrap();
        std::fs::create_dir_all(linked_log.join("meta-dev")).unwrap();
        std::os::unix::fs::symlink(real_dir.join("log.jsonl"), linked_log.join("meta-dev").join("log.jsonl")).unwrap();
        let real = |chosen: &std::path::Path| data_dir_from(Some(chosen.to_path_buf()), Some(home.clone())).unwrap().1;
        let decided = [real(&linked_dir), real(&linked_log)];
        std::fs::remove_dir_all(&home).unwrap();
        assert_eq!(decided, [true, true]);
    }

    #[test]
    fn a_log_locked_past_the_deadline_is_a_port_failure() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("meta-dev-deadline-{nanos}.jsonl"));
        let holder = File::create(&path).unwrap();
        holder.lock().unwrap();
        let mut log = JsonlLog { patience: Duration::from_millis(50), ..JsonlLog::at(path.clone()) };
        let waited = log.transact(|_| (Vec::new(), ()));
        drop(holder);
        std::fs::remove_file(&path).unwrap();
        assert!(waited.unwrap_err().to_string().contains("stayed locked for 50 ms"));
    }
}
