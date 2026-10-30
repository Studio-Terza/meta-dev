use crate::{Log, PortError};
use meta_core::Record;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct JsonlLog {
    path: Result<PathBuf, PortError>,
}

impl JsonlLog {
    pub fn in_data_dir() -> Self {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share")));
        let path = data
            .map(|data| data.join("meta-dev").join("log.jsonl"))
            .ok_or_else(|| PortError::new("neither XDG_DATA_HOME nor HOME is set; the log has no home"));
        Self { path }
    }

    pub fn at(path: PathBuf) -> Self {
        Self { path: Ok(path) }
    }

    fn locked(&self) -> Result<(File, &Path), PortError> {
        let path = self.path.as_ref().map_err(Clone::clone)?;
        path.parent().map_or(Ok(()), fs::create_dir_all).map_err(|error| Self::failed(path, error))?;
        let file = OpenOptions::new().create(true).read(true).append(true).open(path);
        let file = file.and_then(|file| file.lock().map(|()| file)).map_err(|error| Self::failed(path, error))?;
        Ok((file, path))
    }

    fn failed(path: &Path, error: std::io::Error) -> PortError {
        PortError::new(format!("{}: {error}", path.display()))
    }

    fn parse(path: &Path, text: &str) -> Result<Vec<Record>, PortError> {
        text.lines()
            .enumerate()
            .map(|(index, line)| {
                serde_json::from_str(line)
                    .map_err(|error| PortError::new(format!("{} line {}: {error}", path.display(), index + 1)))
            })
            .collect()
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
        Self::parse(path, &text)
    }
}

impl Log for JsonlLog {
    fn read(&self) -> Result<Vec<Record>, PortError> {
        let (mut file, path) = self.locked()?;
        Self::contents(&mut file, path)
    }

    fn append(&mut self, records: &[Record]) -> Result<(), PortError> {
        let (mut file, path) = self.locked()?;
        Self::write(&mut file, path, records)
    }

    fn transact<T>(&mut self, decide: impl FnOnce(Vec<Record>) -> (Vec<Record>, T)) -> Result<T, PortError> {
        let (mut file, path) = self.locked()?;
        let (records, decided) = decide(Self::contents(&mut file, path)?);
        Self::write(&mut file, path, &records).map(|()| decided)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonlLog;
    use crate::Log;
    use std::fs::{File, TryLockError};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn a_transaction_holds_the_log_exclusively_from_read_to_append() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("meta-dev-lock-{nanos}.jsonl"));
        let mut log = JsonlLog::at(path.clone());
        let blocked = log
            .transact(|records| {
                let other = File::open(&path).unwrap();
                (Vec::new(), (records.len(), matches!(other.try_lock(), Err(TryLockError::WouldBlock))))
            })
            .unwrap();
        let free = File::open(&path).unwrap().try_lock().is_ok();
        std::fs::remove_file(&path).unwrap();
        assert_eq!((blocked, free), ((0, true), true));
    }
}
