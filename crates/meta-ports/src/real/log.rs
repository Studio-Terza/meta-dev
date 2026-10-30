use crate::{Log, PortError};
use meta_core::Record;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

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

    fn path(&self) -> Result<&PathBuf, PortError> {
        self.path.as_ref().map_err(Clone::clone)
    }
}

impl Log for JsonlLog {
    fn read(&self) -> Result<Vec<Record>, PortError> {
        let path = self.path()?;
        let text = match fs::read_to_string(path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
            read => read.map_err(|error| PortError::new(format!("{}: {error}", path.display()))),
        }?;
        text.lines()
            .enumerate()
            .map(|(index, line)| {
                serde_json::from_str(line)
                    .map_err(|error| PortError::new(format!("{} line {}: {error}", path.display(), index + 1)))
            })
            .collect()
    }

    fn append(&mut self, records: &[Record]) -> Result<(), PortError> {
        let path = self.path()?;
        let failed = |error: std::io::Error| PortError::new(format!("{}: {error}", path.display()));
        let lines = records
            .iter()
            .map(|record| serde_json::to_string(record).map(|line| line + "\n"))
            .collect::<Result<String, _>>()
            .map_err(|error| PortError::new(error.to_string()))?;
        path.parent().map_or(Ok(()), fs::create_dir_all).map_err(failed)?;
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(lines.as_bytes()))
            .map_err(failed)
    }
}
