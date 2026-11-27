mod ignored;

use std::io;
use std::path::{Path, PathBuf};
use test_failure::Failed;

use file_ports::{
    Batched, Bytes, Disk, FileFailure, Files, FilesFiller, Kind, Memory, Offset, Size, Stamp, Stamps, Tail, Text,
};
use invariants::Ignoring;
use newtype::Wrapping;
use port_core::Run;

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

    fn missing(&self) -> Vec<String> {
        let absent = self.at("absent.txt");
        vec![
            Self::named(&self.files.read(&absent).map(|_| ())),
            Self::named(&self.files.read_text(&absent).map(|_| ())),
            Self::named(&self.files.entries(&absent).map(|_| ())),
            Self::named(&self.files.canonical(&absent).map(|_| ())),
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

    fn canonical_file(&self) -> Result<PathBuf, FileFailure> {
        let path = self.at("here.txt");
        self.files.write(&path, Bytes::from("here"))?;
        assert_eq!(self.files.kind(&path), Kind::File);
        self.files.canonical(&path)
    }
}

struct Checks;

impl Checks {
    fn on(filler: impl FilesFiller + Sync, root: &Path) -> Result<(), Failed> {
        let run = Run::root();
        let files = run.port(filler);
        let contract = Contract { files: &files, root: root.to_path_buf() };
        assert_eq!(contract.written_then_read()?.as_ref(), "written");
        assert_eq!(contract.appended()?, Text::new("one two".to_owned()));
        assert_eq!(contract.missing(), ["missing", "missing", "missing", "missing", "missing", "Missing"]);
        assert_eq!(contract.not_text(), "failed");
        assert_eq!(contract.canonical_file()?.file_name(), Some(std::ffi::OsStr::new("here.txt")));
        Batches::on(&files, root)?;
        run.finish().settled()?;
        Ok(())
    }
}

#[test]
fn disk_and_memory_keep_the_contract_and_answer_batches_path_by_path() -> Result<(), Failed> {
    let disk = tempfile::tempdir()?;
    let memory: Memory = [(PathBuf::from("/scratch/seed"), Bytes::from(""))].into_iter().collect();
    Checks::on(Disk, disk.path())?;
    Checks::on(memory, Path::new("/scratch"))
}

#[test]
fn memory_starts_from_its_entries() -> Result<(), Failed> {
    let run = Run::root();
    let seeded: Memory = [(PathBuf::from("/a/b.txt"), Bytes::from("b"))].into_iter().collect();
    let memory = run.port(seeded);
    assert_eq!(memory.kind(Path::new("/a")), Kind::Directory);
    assert_eq!(memory.entries(Path::new("/a"))?, [PathBuf::from("/a/b.txt")]);
    assert_eq!(memory.read_text(Path::new("/a/b.txt"))?, Text::new("b".to_owned()));
    memory.write(Path::new("top.txt"), Bytes::from("top"))?;
    assert_eq!(memory.kind(Path::new("top.txt")), Kind::File);
    run.finish().settled()?;
    Ok(())
}

#[test]
fn disk_names_a_link() -> Result<(), Failed> {
    let scratch = tempfile::tempdir()?;
    let run = Run::root();
    let disk = run.port(Disk);
    let target = scratch.path().join("target.txt");
    disk.write(&target, Bytes::from("t"))?;
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&target, &link)?;
    assert_eq!(disk.kind(&link), Kind::Link);
    run.finish().settled()?;
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

struct Batches;

impl Batches {
    fn on(files: &impl Files, root: &Path) -> Result<(), FileFailure> {
        let [a, absent] = ["a.txt", "absent.txt"].map(|name| root.join(name));
        files.write(&a, Bytes::from("a"))?;
        assert_eq!(files.kinds(&[a, absent.clone()]), [Kind::File, Kind::Missing]);
        let listed = files.entries_many(&[root.to_path_buf(), absent]);
        assert!(matches!(listed.as_slice(), [Ok(_), Err(FileFailure::Missing(_))]));
        let [deep, single] = ["deep.txt", "c.txt"].map(|name| root.join(name));
        files.write(&deep, Bytes::from("12345"))?;
        files.write(&single, Bytes::from("c"))?;
        let sized = files.sizes(&[deep.clone(), root.join("nothing-here"), single]);
        assert!(
            matches!(sized.as_slice(), [Ok(five), Err(FileFailure::Missing(_)), Ok(one)] if *five == Size::new(5) && *one == Size::new(1))
        );
        let tails: Vec<Tail> = [3, 1]
            .into_iter()
            .filter_map(|at| Offset::try_new(at).ignoring(Ignored::Negative))
            .map(|at| Tail::new(deep.clone(), at))
            .collect();
        assert_eq!(
            files.read_tails(&tails).into_iter().collect::<Result<Vec<Bytes>, _>>()?,
            [Bytes::from("45"), Bytes::from("2345")]
        );
        Ok(())
    }
}

const WRITTEN_AFTER: i64 = 1_767_225_600;

#[test]
fn disk_stamps_each_file_with_the_time_it_was_written_and_misses_an_absent_one() -> Result<(), Failed> {
    let scratch = tempfile::tempdir()?;
    let run = Run::root();
    let disk = run.port(Disk);
    let written = scratch.path().join("stamped.txt");
    disk.write(&written, Bytes::from("now"))?;
    let stamped = disk.stamps_many(&[written, scratch.path().join("absent.txt")]);
    let dated =
        |stamps: &Stamps| [Stamp::Born, Stamp::Touched].map(|stamp| stamps.dated(stamp).into_inner() > WRITTEN_AFTER);
    assert!(matches!(stamped.as_slice(), [Ok(stamps), Err(FileFailure::Missing(_))] if dated(stamps) == [true, true]));
    run.finish().settled()?;
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
    let run = Run::root();
    let seeded: Memory = [(PathBuf::from("/scratch/seed"), Bytes::from(""))].into_iter().collect();
    let (disk, memory) = (run.port(Disk), run.port(seeded));
    let offsets = (0..16).map(Offset::try_new).collect::<Result<Vec<Offset>, _>>()?;
    let read = |at| [Tails::of(&disk, scratch.path(), at), Tails::of(&memory, Path::new("/scratch"), at)];
    let tails: Vec<[Option<Bytes>; 2]> = offsets.into_iter().map(read).collect();
    let wanted: Vec<[Option<Bytes>; 2]> = (0..16_usize)
        .map(|at| [(); 2].map(|()| Some(Bytes::from("0123456789".get(at..).unwrap_or_default()))))
        .collect();
    assert_eq!(tails, wanted);
    drop(memory);
    run.finish().settled()?;
    Ok(())
}
