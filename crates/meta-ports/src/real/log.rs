use crate::{Log, PortError};
use meta_core::Record;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

/// meta-dev's data directory: the log, and the cache of `self/` files last read from main.
pub(super) fn home() -> Result<PathBuf, PortError> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share")));
    data.map(|data| data.join("meta-dev"))
        .ok_or_else(|| PortError::new("neither XDG_DATA_HOME nor HOME is set; meta-dev has no data directory"))
}

#[derive(Debug)]
pub struct JsonlLog {
    path: Result<PathBuf, PortError>,
    patience: Duration,
}

impl JsonlLog {
    pub const PATIENCE: Duration = Duration::from_secs(3);
    const RETRY: Duration = Duration::from_millis(20);

    pub fn in_data_dir() -> Self {
        Self { path: home().map(|home| home.join("log.jsonl")), patience: Self::PATIENCE }
    }

    pub fn at(path: PathBuf) -> Self {
        Self { path: Ok(path), patience: Self::PATIENCE }
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
    use super::JsonlLog;
    use crate::Log;
    use meta_testkit::Scratch;
    use std::fs::{File, TryLockError};
    use std::time::Duration;

    #[test]
    fn a_transaction_holds_the_log_exclusively_from_read_to_append() {
        let scratch = Scratch::new("lock");
        let path = scratch.join("log.jsonl");
        let mut log = JsonlLog::at(path.clone());
        let blocked = log
            .transact(|records| {
                let other = File::open(&path).unwrap();
                (Vec::new(), (records.unwrap().len(), matches!(other.try_lock(), Err(TryLockError::WouldBlock))))
            })
            .unwrap();
        let free = JsonlLog::at(path.clone()).transact(|records| (records.unwrap(), ())).is_ok();
        assert_eq!((blocked, free), ((0, true), true));
    }

    #[test]
    fn a_log_locked_past_the_deadline_is_a_port_failure() {
        let scratch = Scratch::new("deadline");
        let path = scratch.join("log.jsonl");
        let holder = File::create(&path).unwrap();
        holder.lock().unwrap();
        let mut log = JsonlLog { patience: Duration::from_millis(50), ..JsonlLog::at(path.clone()) };
        let waited = log.transact(|_| (Vec::new(), ()));
        drop(holder);
        assert!(waited.unwrap_err().to_string().contains("stayed locked for 50 ms"));
    }
}
