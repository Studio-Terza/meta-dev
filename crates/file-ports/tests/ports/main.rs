mod ignored;

use std::io;
use std::path::{Path, PathBuf};
use test_failure::Failed;

use file_ports::{Absent, Bytes, Disk, FileFailure, Files, Kind, Memory, Offset, Size, Stamp, Stamps, Text};
use invariants::Ignoring;
use newtype::Wrapping;

use ignored::Ignored;

struct Contract<'files, Port: Files> {
    files: &'files Port,
    root: PathBuf,
}

impl<Port: Files> Contract<'_, Port> {
    fn at(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn written_then_read(&self) -> Result<Text, FileFailure> {
        let path = self.at("note.txt");
        self.files.write(&path, Bytes::from("written"))?;
        assert_eq!(self.files.read(&path)?, Bytes::from("written"));
        self.files.read_text(&path)
    }

    fn appended(&self) -> Result<Text, FileFailure> {
        let path = self.at("log.txt");
        self.files.append(&path, &Bytes::from("one "))?;
        self.files.append(&path, &Bytes::from("two"))?;
        self.files.read_text(&path)
    }

    fn copied(&self) -> Result<Text, FileFailure> {
        self.files.write(&self.at("from.txt"), Bytes::from("copied"))?;
        self.files.copy(&self.at("from.txt"), &self.at("to.txt"))?;
        self.files.read_text(&self.at("to.txt"))
    }

    fn nested(&self) -> Result<Vec<PathBuf>, FileFailure> {
        let directory = self.at("outer/inner");
        self.files.create_directories(&directory)?;
        self.files.write(&directory.join("b.txt"), Bytes::from("b"))?;
        self.files.write(&self.at("outer/a.txt"), Bytes::from("a"))?;
        assert_eq!(self.files.kind(&directory), Kind::Directory);
        let mut entries = self.files.entries(&self.at("outer"))?;
        entries.sort();
        Ok(entries)
    }

    fn missing(&self) -> Vec<String> {
        let absent = self.at("absent.txt");
        vec![
            Self::named(&self.files.read(&absent).map(|_| ())),
            Self::named(&self.files.read_text(&absent).map(|_| ())),
            Self::named(&self.files.entries(&absent).map(|_| ())),
            Self::named(&self.files.canonical(&absent).map(|_| ())),
            Self::named(&self.files.copy(&absent, &self.at("copy.txt"))),
            Self::named(&self.files.write(&self.at("absent/inside.txt"), Bytes::from("x"))),
            format!("{:?}", self.files.kind(&absent)),
        ]
    }

    fn named(result: &Result<(), FileFailure>) -> String {
        match result {
            Ok(()) => "ok".to_owned(),
            Err(FileFailure::Missing(_)) => "missing".to_owned(),
            Err(FileFailure::Failed(_)) => "failed".to_owned(),
        }
    }

    fn not_text(&self) -> String {
        let path = self.at("binary");
        let written = self.files.write(&path, Bytes::new(vec![0xff, 0xfe]));
        Self::named(&written.and_then(|()| self.files.read_text(&path).map(|_| ())))
    }

    fn beneath_a_file(&self) -> String {
        let written = self.files.write(&self.at("blocker"), Bytes::from(""));
        Self::named(&written.and_then(|()| self.files.create_directories(&self.at("blocker/inner"))))
    }

    fn resolved(&self) -> Result<[PathBuf; 2], FileFailure> {
        let real = self.files.canonical(&self.at("outer/a.txt"))?;
        let dotted = self.files.canonical(&self.at("outer/./inner/../a.txt"))?;
        assert_eq!(Self::named(&self.files.read(&self.at("outer")).map(|_| ())), "failed");
        Ok([real, dotted])
    }

    fn canonical_file(&self) -> Result<PathBuf, FileFailure> {
        let path = self.at("here.txt");
        self.files.write(&path, Bytes::from("here"))?;
        assert_eq!(self.files.kind(&path), Kind::File);
        self.files.canonical(&path)
    }
}

struct Checks;

impl Checks {
    fn on(files: &(impl Files + Sync), root: &Path) -> Result<(), FileFailure> {
        let contract = Contract { files, root: root.to_path_buf() };
        assert_eq!(contract.written_then_read()?.as_ref(), "written");
        assert_eq!(contract.appended()?, Text::new("one two".to_owned()));
        assert_eq!(contract.copied()?, Text::new("copied".to_owned()));
        assert_eq!(contract.nested()?, vec![contract.at("outer/a.txt"), contract.at("outer/inner")]);
        let [real, dotted] = std::thread::scope(|scope| scope.spawn(|| contract.resolved()).join())
            .ignoring(Ignored::Panicked)
            .unwrap_or_else(|| Err(FileFailure::Failed(io::Error::from(io::ErrorKind::Other))))?;
        assert_eq!(dotted, real);
        assert_eq!(contract.missing(), ["missing", "missing", "missing", "missing", "missing", "missing", "Missing"]);
        assert_eq!(contract.not_text(), "failed");
        assert_eq!(contract.beneath_a_file(), "failed");
        assert_eq!(contract.canonical_file()?.file_name(), Some(std::ffi::OsStr::new("here.txt")));
        Ok(())
    }
}

impl Checks {
    fn kept(files: &(impl Files + Sync), root: &Path) -> Result<(), FileFailure> {
        files.create_directories(&root.join("contract"))?;
        files.create_directories(&root.join("batch"))?;
        Checks::on(files, &root.join("contract"))?;
        Batches::on(files, &root.join("batch"))
    }
}

#[test]
fn disk_and_the_fake_keep_the_contract_and_answer_path_by_path() -> Result<(), FileFailure> {
    let disk = tempfile::tempdir().map_err(FileFailure::from)?;
    Checks::kept(&Disk, disk.path())?;
    Checks::kept(&Memory::default(), Path::new("/scratch"))
}

#[test]
fn fake_starts_from_its_entries() -> Result<(), FileFailure> {
    let memory: Memory = [(PathBuf::from("/a/b.txt"), Bytes::from("b"))].into_iter().collect();
    assert_eq!(memory.kind(Path::new("/a")), Kind::Directory);
    assert_eq!(memory.entries(Path::new("/a"))?, [PathBuf::from("/a/b.txt")]);
    assert_eq!(memory.read_text(Path::new("/a/b.txt"))?, Text::new("b".to_owned()));
    memory.write(Path::new("top.txt"), Bytes::from("top"))?;
    assert_eq!(memory.kind(Path::new("top.txt")), Kind::File);
    let empty = Memory::default();
    empty.write(Path::new("alone.txt"), Bytes::from("alone"))?;
    assert_eq!(empty.kind(Path::new("alone.txt")), Kind::File);
    Ok(())
}

#[test]
fn disk_names_a_link() -> Result<(), FileFailure> {
    let scratch = tempfile::tempdir().map_err(FileFailure::from)?;
    let target = scratch.path().join("target.txt");
    Disk.write(&target, Bytes::from("t"))?;
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&target, &link).map_err(FileFailure::from)?;
    assert_eq!(Disk.kind(&link), Kind::Link);
    Ok(())
}

#[test]
fn a_failure_gives_back_its_error() {
    let missing = io::Error::from(FileFailure::from(io::Error::from(io::ErrorKind::NotFound)));
    let denied = io::Error::from(FileFailure::from(io::Error::from(io::ErrorKind::PermissionDenied)));
    assert_eq!((missing.kind(), denied.kind()), (io::ErrorKind::NotFound, io::ErrorKind::PermissionDenied));
    assert_eq!(
        FileFailure::from(io::Error::from(io::ErrorKind::NotFound)).to_string(),
        io::Error::from(io::ErrorKind::NotFound).to_string()
    );
}

#[test]
fn absent_holds_nothing_and_takes_nothing() {
    let path = Path::new("/a.txt");
    assert!(matches!(Absent.read(path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.read_text(path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.write(path, Bytes::from("a")), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.append(path, &Bytes::from("a")), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.create_directories(path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.copy(path, path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.entries(path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.canonical(path), Err(FileFailure::Missing(_))));
    assert!(matches!(Absent.size(path), Err(FileFailure::Missing(_))));
    assert_eq!(Absent.kind(path), Kind::Missing);
}

struct Batches;

impl Batches {
    fn on(files: &impl Files, root: &Path) -> Result<(), FileFailure> {
        files.write(&root.join("deep.txt"), Bytes::from("12345"))?;
        assert_eq!(files.size(&root.join("deep.txt"))?, Size::new(5));
        assert!(matches!(files.size(&root.join("nothing-here")), Err(FileFailure::Missing(_))));
        Ok(())
    }
}

const WRITTEN_AFTER: i64 = 1_767_225_600;

#[test]
fn disk_stamps_each_file_with_the_time_it_was_written_and_misses_an_absent_one() -> Result<(), FileFailure> {
    let scratch = tempfile::tempdir().map_err(FileFailure::from)?;
    let written = scratch.path().join("stamped.txt");
    Disk.write(&written, Bytes::from("now"))?;
    let stamped = [Disk.stamps(&written), Disk.stamps(&scratch.path().join("absent.txt"))];
    let dated =
        |stamps: &Stamps| [Stamp::Born, Stamp::Touched].map(|stamp| stamps.dated(stamp).into_inner() > WRITTEN_AFTER);
    assert!(matches!(stamped.as_slice(), [Ok(stamps), Err(FileFailure::Missing(_))] if dated(stamps) == [true, true]));
    Ok(())
}

struct Tails;

impl Tails {
    fn of(files: &impl Files, root: &Path, offset: Offset) -> Option<Bytes> {
        let path = root.join("tail.txt");
        files
            .write(&path, Bytes::from("0123456789"))
            .and_then(|()| files.read_from(&path, offset))
            .ignoring(Ignored::Failing)
    }
}

#[test]
fn read_from_returns_the_bytes_after_the_offset_alike_on_disk_and_in_memory() -> Result<(), Failed> {
    let scratch = tempfile::tempdir()?;
    let memory = Memory::default();
    memory.create_directories(Path::new("/scratch"))?;
    let offsets = (0..16).map(Offset::try_new).collect::<Result<Vec<Offset>, _>>()?;
    let read = |at| [Tails::of(&Disk, scratch.path(), at), Tails::of(&memory, Path::new("/scratch"), at)];
    let tails: Vec<[Option<Bytes>; 2]> = offsets.into_iter().map(read).collect();
    let wanted: Vec<[Option<Bytes>; 2]> = (0..16_usize)
        .map(|at| [(); 2].map(|()| Some(Bytes::from("0123456789".get(at..).unwrap_or_default()))))
        .collect();
    assert_eq!(tails, wanted);
    Ok(())
}
