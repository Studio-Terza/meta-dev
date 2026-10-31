use crate::{Log, PortError};
use meta_core::{Head, Marking, Place, Record};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

/// meta-dev's data directory: the log, and the cache of `self/` files last read from main.
pub(super) fn home() -> Result<PathBuf, PortError> {
    data_dir().map(|(home, _)| home)
}

/// The data directory, and whether it is the default one, `$HOME/.local/share/meta-dev`. Which log
/// is real is decided by its first record (`Head`), not here: the place matters only for a default
/// log not yet marked.
fn data_dir() -> Result<(PathBuf, Place), PortError> {
    data_dir_from(std::env::var_os("XDG_DATA_HOME").map(PathBuf::from), std::env::var_os("HOME").map(PathBuf::from))
}

fn data_dir_from(chosen: Option<PathBuf>, home: Option<PathBuf>) -> Result<(PathBuf, Place), PortError> {
    match (chosen, home) {
        (Some(chosen), _) => Ok((chosen.join(DIR), Place::Elsewhere)),
        (None, Some(home)) => Ok((home.join(".local").join("share").join(DIR), Place::Default)),
        (None, None) => Err(PortError::new("neither XDG_DATA_HOME nor HOME is set; meta-dev has no data directory")),
    }
}

/// meta-dev's directory under the data home, and the log in it.
const DIR: &str = "meta-dev";
const LOG: &str = "log.jsonl";
/// Beside the log, the copy `mark` keeps of the log as it was before marking.
const UNMARKED: &str = "log.jsonl.unmarked";

#[derive(Debug)]
pub struct JsonlLog {
    path: Result<PathBuf, PortError>,
    patience: Duration,
    place: Place,
}

impl JsonlLog {
    pub const PATIENCE: Duration = Duration::from_secs(3);
    const RETRY: Duration = Duration::from_millis(20);

    /// The log in the data directory. With no data directory its first record cannot be read, so
    /// it is taken as real: fail closed.
    pub fn in_data_dir() -> Self {
        Self::in_dir(data_dir())
    }

    fn in_dir(dir: Result<(PathBuf, Place), PortError>) -> Self {
        let (path, place) = match dir {
            Ok((home, place)) => (Ok(home.join(LOG)), place),
            Err(error) => (Err(error), Place::Default),
        };
        Self { path, patience: Self::PATIENCE, place }
    }

    /// The log at `path`, outside the default data dir: real only when its first record is the marker.
    pub fn at(path: PathBuf) -> Self {
        Self { path: Ok(path), patience: Self::PATIENCE, place: Place::Elsewhere }
    }

    /// The log's first record, read through whatever path, symlink or hard link reaches it.
    fn head(&self) -> Head {
        match self.path.as_ref().map(|path| (path, File::open(path))) {
            Err(_) => Head::Unreadable,
            Ok((_, Err(error))) if error.kind() == ErrorKind::NotFound => Head::Empty,
            Ok((_, Err(_))) => Head::Unreadable,
            Ok((path, Ok(file))) => Self::head_of(path, BufReader::new(file)),
        }
    }

    /// The first record of what `reader` holds.
    fn head_of(path: &Path, mut reader: impl BufRead) -> Head {
        let mut line = String::new();
        let first = reader
            .read_line(&mut line)
            .map_err(|error| Self::failed(path, error))
            .and_then(|_| super::jsonl::records(path, &line));
        Head::of(first.as_ref().map(|records| records.first()))
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
        self.head().is_real(self.place)
    }

    /// Keeps the log as it was in `log.jsonl.unmarked` first, and refuses when that copy already
    /// exists, so that a marking cut short never loses the earlier copy.
    fn mark(&mut self, marker: Record) -> Result<Marking, PortError> {
        let (mut file, path) = self.locked()?;
        let mut text = String::new();
        file.read_to_string(&mut text).map_err(|error| Self::failed(path, error))?;
        if Self::head_of(path, text.as_bytes()) == Head::Marked {
            return Ok(Marking::AlreadyMarked);
        }
        let unmarked = path.with_file_name(UNMARKED);
        let kept = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&unmarked)
            .and_then(|mut copy| copy.write_all(text.as_bytes()).and_then(|()| copy.sync_all()));
        kept.map_err(|error| Self::failed(&unmarked, error))?;
        file.set_len(0).map_err(|error| Self::failed(path, error))?;
        Self::write(&mut file, path, &[marker])?;
        file.write_all(text.as_bytes()).and_then(|()| file.sync_all()).map_err(|error| Self::failed(path, error))?;
        Ok(Marking::Added)
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
    use meta_core::{Event, Marking, Record, Timestamp};
    use meta_testkit::Scratch;
    use std::fs::{self, File, TryLockError};
    use std::path::Path;
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

    const MARKER: &str = "{\"at\":1,\"event\":\"RealLog\"}\n";
    const DONE: &str = "{\"at\":2,\"event\":\"Done\",\"task\":1}\n";

    /// The log meta reaches with XDG_DATA_HOME and HOME set so.
    fn reached(xdg: Option<&Path>, home: Option<&Path>) -> JsonlLog {
        JsonlLog::in_dir(data_dir_from(xdg.map(Path::to_path_buf), home.map(Path::to_path_buf)))
    }

    #[test]
    fn a_marked_log_is_real_through_another_home_a_hard_link_a_symlink_or_with_home_unset() {
        let root = Scratch::new("marked");
        let (lead, other) = (root.join("lead"), root.join("other"));
        let share = lead.join(".local").join("share");
        let real_dir = share.join("meta-dev");
        fs::create_dir_all(&real_dir).unwrap();
        fs::write(real_dir.join("log.jsonl"), [MARKER, DONE].concat()).unwrap();
        let (hard, dir_link, log_link) = (root.join("hard"), root.join("dir-link"), root.join("log-link"));
        [&hard, &dir_link, &log_link].iter().for_each(|dir| fs::create_dir_all(dir).unwrap());
        fs::create_dir_all(hard.join("meta-dev")).unwrap();
        fs::hard_link(real_dir.join("log.jsonl"), hard.join("meta-dev").join("log.jsonl")).unwrap();
        std::os::unix::fs::symlink(&real_dir, dir_link.join("meta-dev")).unwrap();
        fs::create_dir_all(log_link.join("meta-dev")).unwrap();
        std::os::unix::fs::symlink(real_dir.join("log.jsonl"), log_link.join("meta-dev").join("log.jsonl")).unwrap();
        let reach = [
            reached(None, Some(&lead)),
            reached(Some(&share), Some(&other)),
            reached(Some(&hard), Some(&other)),
            reached(Some(&dir_link), Some(&other)),
            reached(Some(&log_link), Some(&other)),
            reached(Some(&share), None),
            reached(None, None),
        ];
        let real = reach.map(|log| log.is_real());
        assert_eq!(real, [true; 7]);
    }

    #[test]
    fn an_unmarked_log_anywhere_else_is_scratch_and_the_default_one_is_real_until_marked() {
        let root = Scratch::new("unmarked");
        let firsts = ["", DONE, "not a record\n"];
        let logs = firsts.map(|first| {
            let dir = root.join(format!("scratch-{}", first.len()));
            fs::create_dir_all(dir.join("meta-dev")).unwrap();
            fs::write(dir.join("meta-dev").join("log.jsonl"), [first, DONE].concat()).unwrap();
            reached(Some(&dir), Some(&root))
        });
        let missing = reached(Some(&root.join("missing")), Some(&root));
        let home = root.join("home");
        let default_missing = reached(None, Some(&home));
        fs::create_dir_all(home.join(".local").join("share").join("meta-dev")).unwrap();
        fs::write(home.join(".local").join("share").join("meta-dev").join("log.jsonl"), DONE).unwrap();
        let default_unmarked = reached(None, Some(&home));
        let real = [logs.map(|log| log.is_real()).to_vec(), vec![missing.is_real()]].concat();
        let default = [default_missing.is_real(), default_unmarked.is_real()];
        assert_eq!(real, [false, false, true, false], "empty first line, unmarked, garbled, missing");
        assert_eq!(default, [true, true]);
    }

    #[test]
    fn marking_puts_the_marker_first_keeps_the_unmarked_copy_and_happens_once() {
        let root = Scratch::new("mark");
        let path = root.join("log.jsonl");
        fs::write(&path, [DONE, DONE].concat()).unwrap();
        fs::hard_link(&path, root.join("linked.jsonl")).unwrap();
        let marker = Record { at: Timestamp::from_unix_seconds(1), event: Event::RealLog };
        let mut log = JsonlLog::at(path.clone());
        let before = log.is_real();
        let marked = [log.mark(marker.clone()).unwrap(), log.mark(marker.clone()).unwrap()];
        let text = fs::read_to_string(&path).unwrap();
        let kept = fs::read_to_string(root.join("log.jsonl.unmarked")).unwrap();
        let after = [log.is_real(), JsonlLog::at(root.join("linked.jsonl")).is_real()];
        let records = log.read().unwrap().len();
        let other = root.join("other.jsonl");
        fs::write(&other, DONE).unwrap();
        fs::write(root.join("log.jsonl.unmarked"), "earlier copy").unwrap();
        let refused = JsonlLog::at(other.clone()).mark(marker).unwrap_err().to_string();
        let untouched = fs::read_to_string(&other).unwrap();
        assert_eq!((before, marked), (false, [Marking::Added, Marking::AlreadyMarked]));
        assert_eq!((text, kept), ([MARKER, DONE, DONE].concat(), [DONE, DONE].concat()));
        assert_eq!((after, records), ([true, true], 3));
        assert!(refused.contains("log.jsonl.unmarked"), "{refused}");
        assert_eq!(untouched, DONE);
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
