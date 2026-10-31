use crate::{Log, PortError};
use meta_core::{Head, Marking, Place, Record, SideLine, Standing, UnverifiedEscape, UnverifiedEscapes};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
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
/// Beside the log, the marked log while `mark` writes it, before it is renamed over the log.
const MARKING: &str = "log.jsonl.marking";
/// Beside the log, the escapes from builds not verified to be from main.
const SIDE: &str = "escapes.unverified.jsonl";
/// Beside the side file, the file whose lock orders appends to the side file against an import. It
/// is never renamed or replaced, so the lock always names the same file.
const SIDE_LOCK: &str = "escapes.unverified.lock";
/// Beside the side file, the lines it keeps after an import, while they are written before they are
/// renamed over the side file.
const SIDE_IMPORTED: &str = "escapes.unverified.jsonl.importing";

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
        let mut line = Vec::new();
        let first = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| Self::failed(path, error))
            .and_then(|_| {
                String::from_utf8(line)
                    .map_err(|error| Self::failed(path, std::io::Error::new(ErrorKind::InvalidData, error)))
            })
            .and_then(|line| super::jsonl::records(path, &line));
        Head::of(first.as_ref().map(|records| records.first()))
    }

    /// The log, opened and locked. `mark` renames a new file over the log, so a process that waited
    /// on the lock may hold the old file: it opens the log again, or its append would be lost.
    fn locked(&self) -> Result<(File, &Path), PortError> {
        let path = self.path.as_ref().map_err(Clone::clone)?;
        path.parent().map_or(Ok(()), fs::create_dir_all).map_err(|error| Self::failed(path, error))?;
        let deadline = Instant::now() + self.patience;
        loop {
            let file = OpenOptions::new().create(true).read(true).append(true).open(path);
            let file = file.map_err(|error| Self::failed(path, error))?;
            self.lock(&file, path, deadline)?;
            if Self::is_current(&file, path) {
                return Ok((file, path));
            }
        }
    }

    fn lock(&self, file: &File, path: &Path, deadline: Instant) -> Result<(), PortError> {
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(()),
                Err(TryLockError::WouldBlock) if Instant::now() < deadline => thread::sleep(Self::RETRY),
                Err(TryLockError::WouldBlock) => {
                    let waited = self.patience.as_millis();
                    return Err(PortError::new(format!("{} stayed locked for {waited} ms", path.display())));
                }
                Err(TryLockError::Error(error)) => return Err(Self::failed(path, error)),
            }
        }
    }

    /// The side file's lock, taken within the same patience as the log's. A writer that cannot get
    /// it fails visibly: the escape is not dropped.
    fn side_locked(&self, side: &Path) -> Result<File, PortError> {
        let path = side.with_file_name(SIDE_LOCK);
        side.parent().map_or(Ok(()), fs::create_dir_all).map_err(|error| Self::failed(&path, error))?;
        let file =
            OpenOptions::new().create(true).append(true).open(&path).map_err(|error| Self::failed(&path, error))?;
        self.lock(&file, &path, Instant::now() + self.patience)?;
        Ok(file)
    }

    /// Whether `file` is still the file at `path`.
    fn is_current(file: &File, path: &Path) -> bool {
        match (file.metadata(), fs::metadata(path)) {
            (Ok(held), Ok(there)) => (held.dev(), held.ino()) == (there.dev(), there.ino()),
            _ => false,
        }
    }

    /// The file `path` reaches, through any symlinks, or `path` itself while nothing is there.
    fn resolved(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_owned())
    }

    /// The side file, beside the file the log's path reaches, so that a symlink leads to the same
    /// side file as the log's own path. A hard link cannot be followed: it leads to its own.
    fn side(&self) -> Result<PathBuf, PortError> {
        self.path.as_ref().map_err(Clone::clone).map(|path| Self::resolved(path).with_file_name(SIDE))
    }

    /// Puts `marker` and `text` at `path` in one step: written and synced beside it first, then
    /// renamed over it, then the rename synced. A crash leaves the old log or the new one, whole.
    fn replace(path: &Path, marker: Record, text: &str) -> Result<(), PortError> {
        let temp = path.with_file_name(MARKING);
        let failed = |error| Self::failed(&temp, error);
        let mut file = File::create(&temp).map_err(failed)?;
        file.set_permissions(fs::metadata(path).map_err(failed)?.permissions()).map_err(failed)?;
        Self::write(&mut file, &temp, &[marker])?;
        file.write_all(text.as_bytes()).and_then(|()| file.sync_all()).map_err(failed)?;
        fs::rename(&temp, path).map_err(failed)?;
        path.parent().map_or(Ok(()), |dir| File::open(dir).and_then(|dir| dir.sync_all())).map_err(failed)
    }

    /// Puts `kept` at `side` in one step: written and synced beside it, then renamed over it, then
    /// the rename synced. A crash leaves the side file as it was or as it becomes, whole.
    fn rewrite_side(side: &Path, kept: &[SideLine]) -> Result<(), PortError> {
        let temp = side.with_file_name(SIDE_IMPORTED);
        let failed = |error| Self::failed(&temp, error);
        let text: Vec<u8> = kept.iter().flat_map(|line| [line.raw(), b"\n"].concat()).collect();
        let mut file = File::create(&temp).map_err(failed)?;
        file.write_all(&text).and_then(|()| file.sync_all()).map_err(failed)?;
        fs::rename(&temp, side).map_err(failed)?;
        side.parent().map_or(Ok(()), |dir| File::open(dir).and_then(|dir| dir.sync_all())).map_err(failed)
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

    fn standing(&self) -> Standing {
        self.head().standing(self.place)
    }

    /// Keeps the log as it was in `log.jsonl.unmarked` first, and refuses when that copy already
    /// exists, so that a marking cut short never loses the earlier copy. The marked log replaces
    /// the log by rename, through any symlink, so a hard link made earlier keeps the old file.
    fn mark(&mut self, marker: Record) -> Result<Marking, PortError> {
        let (mut file, path) = self.locked()?;
        let mut text = String::new();
        file.read_to_string(&mut text).map_err(|error| Self::failed(path, error))?;
        if Self::head_of(path, text.as_bytes()) == Head::Marked {
            return Ok(Marking::AlreadyMarked);
        }
        let path = Self::resolved(path);
        let unmarked = path.with_file_name(UNMARKED);
        let kept = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&unmarked)
            .and_then(|mut copy| copy.write_all(text.as_bytes()).and_then(|()| copy.sync_all()));
        kept.map_err(|error| {
            PortError::new(format!(
                "{}: {error}; the log was not changed. Next: compare that copy with the log and move it aside, then mark again.",
                unmarked.display()
            ))
        })?;
        Self::replace(&path, marker, &text).map_err(|error| {
            PortError::new(format!("marking failed: {error}; the log as it was is in {}", unmarked.display()))
        })?;
        Ok(Marking::Added)
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        let (mut file, path) = self.locked()?;
        Self::contents(&mut file, path)
    }

    fn set_aside(&mut self, escape: &UnverifiedEscape) -> Result<String, PortError> {
        let side = self.side()?;
        let line = serde_json::to_string(escape).map_err(|error| PortError::new(error.to_string()))? + "\n";
        // Held until the line is synced: an import reads and renames the side file under it.
        let _held = self.side_locked(&side)?;
        let written = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&side)
            .and_then(|mut file| file.write_all(line.as_bytes()).and_then(|()| file.sync_all()));
        written.map_err(|error| Self::failed(&side, error)).map(|()| side.display().to_string())
    }

    fn set_aside_escapes(&self) -> Result<UnverifiedEscapes, PortError> {
        let side = self.side()?;
        match fs::read(&side) {
            Ok(bytes) => Ok(super::jsonl::lenient(&bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(UnverifiedEscapes::default()),
            Err(error) => Err(Self::failed(&side, error)),
        }
    }

    fn import_set_aside<T>(
        &mut self,
        decide: impl FnOnce(
            Standing,
            Result<Vec<Record>, PortError>,
            Vec<SideLine>,
        ) -> (Vec<Record>, Option<Vec<SideLine>>, T),
    ) -> Result<T, PortError> {
        let (mut file, path) = self.locked()?;
        let side = self.side()?;
        // From before the side file is read until after it is renamed: an escape set aside meanwhile
        // waits, then lands in the new side file.
        let _held = self.side_locked(&side)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|error| Self::failed(path, error))?;
        let standing = Self::head_of(path, bytes.as_slice()).standing(self.place);
        let text = String::from_utf8(bytes)
            .map_err(|error| Self::failed(path, std::io::Error::new(ErrorKind::InvalidData, error)));
        let lines = match fs::read(&side) {
            Ok(bytes) => super::jsonl::side_lines(&bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(Self::failed(&side, error)),
        };
        let (records, kept, decided) =
            decide(standing, text.and_then(|text| super::jsonl::records(path, &text)), lines.clone());
        Self::write(&mut file, path, &records)?;
        file.sync_all().map_err(|error| Self::failed(path, error))?;
        match kept {
            Some(kept) if kept != lines => Self::rewrite_side(&side, &kept)?,
            _ => {}
        }
        Ok(decided)
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Standing, Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        let (mut file, path) = self.locked()?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|error| Self::failed(path, error))?;
        let standing = Self::head_of(path, bytes.as_slice()).standing(self.place);
        let text = String::from_utf8(bytes)
            .map_err(|error| Self::failed(path, std::io::Error::new(ErrorKind::InvalidData, error)));
        let (records, decided) = decide(standing, text.and_then(|text| super::jsonl::records(path, &text)));
        Self::write(&mut file, path, &records).map(|()| decided)
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonlLog, data_dir_from};
    use crate::Log;
    use meta_core::{Event, Marking, Record, Standing, Timestamp, UnverifiedEscape};
    use meta_testkit::Scratch;
    use std::fs::{self, File, TryLockError};
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn a_transaction_holds_the_log_exclusively_from_read_to_append() {
        let scratch = Scratch::new("lock");
        let path = scratch.join("log.jsonl");
        let mut log = JsonlLog::at(path.clone());
        let blocked = log
            .transact(|_, records| {
                let other = File::open(&path).unwrap();
                (Vec::new(), (records.unwrap().len(), matches!(other.try_lock(), Err(TryLockError::WouldBlock))))
            })
            .unwrap();
        let free = JsonlLog::at(path.clone()).transact(|_, records| (records.unwrap(), ())).is_ok();
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
        let real = reach.map(|log| log.standing().is_real());
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
        let real = [logs.map(|log| log.standing().is_real()).to_vec(), vec![missing.standing().is_real()]].concat();
        let default = [default_missing.standing().is_real(), default_unmarked.standing().is_real()];
        assert_eq!(real, [false, false, true, false], "empty first line, unmarked, garbled, missing");
        assert_eq!(default, [true, true]);
    }

    fn marker() -> Record {
        Record { at: Timestamp::from_unix_seconds(1), event: Event::RealLog }
    }

    #[test]
    fn marking_puts_the_marker_first_keeps_the_unmarked_copy_and_happens_once() {
        let root = Scratch::new("mark");
        let path = root.join("log.jsonl");
        fs::write(&path, [DONE, DONE].concat()).unwrap();
        let mut log = JsonlLog::at(path.clone());
        let before = log.standing().is_real();
        let marked = [log.mark(marker()).unwrap(), log.mark(marker()).unwrap()];
        let text = fs::read_to_string(&path).unwrap();
        let kept = fs::read_to_string(root.join("log.jsonl.unmarked")).unwrap();
        assert_eq!((before, marked), (false, [Marking::Added, Marking::AlreadyMarked]));
        assert_eq!((text, kept), ([MARKER, DONE, DONE].concat(), [DONE, DONE].concat()));
        assert_eq!((log.standing().is_real(), log.read().unwrap().len()), (true, 3));
    }

    #[test]
    fn marking_refuses_when_an_unmarked_copy_exists_and_points_at_it() {
        let root = Scratch::new("mark-copy");
        let path = root.join("log.jsonl");
        fs::write(&path, DONE).unwrap();
        fs::write(root.join("log.jsonl.unmarked"), "earlier copy").unwrap();
        let refused = JsonlLog::at(path.clone()).mark(marker()).unwrap_err().to_string();
        assert!(refused.contains("log.jsonl.unmarked"), "{refused}");
        assert_eq!(fs::read_to_string(&path).unwrap(), DONE);
        assert_eq!(fs::read_to_string(root.join("log.jsonl.unmarked")).unwrap(), "earlier copy");
    }

    #[test]
    fn marking_that_fails_before_the_rename_leaves_the_log_whole_and_points_at_the_unmarked_copy() {
        let root = Scratch::new("mark-cut");
        let path = root.join("log.jsonl");
        fs::write(&path, [DONE, DONE].concat()).unwrap();
        fs::create_dir(root.join("log.jsonl.marking")).unwrap();
        let failed = JsonlLog::at(path.clone()).mark(marker()).unwrap_err().to_string();
        assert!(failed.contains("log.jsonl.unmarked"), "{failed}");
        assert_eq!(fs::read_to_string(&path).unwrap(), [DONE, DONE].concat());
        assert_eq!(fs::read_to_string(root.join("log.jsonl.unmarked")).unwrap(), [DONE, DONE].concat());
    }

    #[test]
    fn marking_replaces_the_log_by_rename_so_a_hard_link_keeps_the_old_file_and_a_symlink_keeps_pointing() {
        let root = Scratch::new("mark-links");
        let (path, hard, soft) = (root.join("log.jsonl"), root.join("hard.jsonl"), root.join("soft.jsonl"));
        fs::write(&path, DONE).unwrap();
        fs::hard_link(&path, &hard).unwrap();
        std::os::unix::fs::symlink(&path, &soft).unwrap();
        JsonlLog::at(soft.clone()).mark(marker()).unwrap();
        assert!(fs::symlink_metadata(&soft).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&path).unwrap(), [MARKER, DONE].concat());
        assert_eq!(fs::read_to_string(&hard).unwrap(), DONE);
        assert!(root.join("log.jsonl.unmarked").exists());
    }

    #[test]
    fn a_writer_waiting_on_the_lock_while_the_log_is_replaced_appends_to_the_new_log() {
        let root = Scratch::new("replaced");
        let path = root.join("log.jsonl");
        fs::write(&path, DONE).unwrap();
        let holder = File::open(&path).unwrap();
        holder.lock().unwrap();
        let mut waiting = JsonlLog::at(path.clone());
        let writer = thread::spawn(move || {
            let record = Record {
                at: Timestamp::from_unix_seconds(2),
                event: Event::Done { task: "task-1".parse().unwrap(), by: None },
            };
            waiting.transact(|_, _| (vec![record], ()))
        });
        thread::sleep(Duration::from_millis(100));
        let replacement = root.join("replacement");
        fs::write(&replacement, [MARKER, DONE].concat()).unwrap();
        fs::rename(&replacement, &path).unwrap();
        drop(holder);
        writer.join().unwrap().unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), [MARKER, DONE, DONE].concat());
    }

    #[test]
    fn a_transaction_reads_the_standing_from_the_first_record_inside_the_lock() {
        let root = Scratch::new("standing-locked");
        let path = root.join("log.jsonl");
        fs::write(&path, DONE).unwrap();
        let mut stale = JsonlLog::at(path.clone());
        let before = stale.standing();
        JsonlLog::at(path.clone()).mark(marker()).unwrap();
        let inside = stale.transact(|standing, _| (Vec::new(), standing)).unwrap();
        assert_eq!((before, inside), (Standing::Scratch, Standing::MarkedElsewhere));
    }

    fn escape_line(reason: &str) -> UnverifiedEscape {
        serde_json::from_str(&format!("{{\"at\":3,\"kind\":\"gap\",\"reason\":\"{reason}\"}}")).unwrap()
    }

    #[test]
    fn escapes_set_aside_during_an_import_each_end_up_in_the_log_or_the_side_file_exactly_once() {
        let root = Scratch::new("side-race");
        let path = root.join("log.jsonl");
        fs::write(&path, MARKER).unwrap();
        let side = root.join("escapes.unverified.jsonl");
        (0..4).for_each(|n| {
            JsonlLog::at(path.clone()).set_aside(&escape_line(&format!("before-{n}"))).unwrap();
        });
        let writers: Vec<_> = (0..4)
            .map(|writer| {
                let mut log = JsonlLog::at(path.clone());
                thread::spawn(move || {
                    (0..10).for_each(|n| {
                        log.set_aside(&escape_line(&format!("w{writer}-{n}"))).unwrap();
                        thread::sleep(Duration::from_millis(15));
                    });
                })
            })
            .collect();
        thread::sleep(Duration::from_millis(40));
        let mut moved = Vec::new();
        JsonlLog::at(path.clone())
            .import_set_aside(|_, _, lines| {
                thread::sleep(Duration::from_millis(250));
                moved = lines.iter().map(|line| line.raw().to_vec()).collect();
                (Vec::new(), Some(Vec::new()), ())
            })
            .unwrap();
        writers.into_iter().for_each(|writer| writer.join().unwrap());
        let left: Vec<Vec<u8>> =
            super::super::jsonl::side_lines(&fs::read(&side).unwrap()).iter().map(|line| line.raw().to_vec()).collect();
        let mut all: Vec<Vec<u8>> = [moved.clone(), left.clone()].concat();
        all.sort();
        let mut wanted: Vec<Vec<u8>> = (0..4)
            .map(|n| format!("before-{n}"))
            .chain((0..4).flat_map(|w| (0..10).map(move |n| format!("w{w}-{n}"))))
            .map(|reason| serde_json::to_vec(&escape_line(&reason)).unwrap())
            .collect();
        wanted.sort();
        assert_eq!(all, wanted);
        assert!(moved.len() >= 4 && !left.is_empty(), "moved {}, left {}", moved.len(), left.len());
    }

    #[test]
    fn setting_an_escape_aside_while_the_side_file_is_locked_past_the_deadline_fails_and_writes_nothing() {
        let root = Scratch::new("side-deadline");
        let path = root.join("log.jsonl");
        let holder = File::create(root.join("escapes.unverified.lock")).unwrap();
        holder.lock().unwrap();
        let mut log = JsonlLog { patience: Duration::from_millis(50), ..JsonlLog::at(path) };
        let refused = log.set_aside(&escape_line("held"));
        drop(holder);
        assert!(refused.unwrap_err().to_string().contains("stayed locked for 50 ms"));
        assert!(!root.join("escapes.unverified.jsonl").exists());
    }

    #[test]
    fn a_log_locked_past_the_deadline_is_a_port_failure() {
        let scratch = Scratch::new("deadline");
        let path = scratch.join("log.jsonl");
        let holder = File::create(&path).unwrap();
        holder.lock().unwrap();
        let mut log = JsonlLog { patience: Duration::from_millis(50), ..JsonlLog::at(path.clone()) };
        let waited = log.transact(|_, _| (Vec::new(), ()));
        drop(holder);
        assert!(waited.unwrap_err().to_string().contains("stayed locked for 50 ms"));
    }
}
