use std::path::PathBuf;

use file_ports::{Absent, Bytes, Cached, Disk, Files, Kind, Memory, Offset};
use newtype::Wrapping;
use proptest::prelude::{any, prop_assert_eq, proptest};

proptest! {
    #[test]
    fn disk_and_memory_read_back_what_was_written(name in "[a-z]{1,8}", bytes in any::<Bytes>()) {
        let scratch = tempfile::tempdir()?;
        let path = scratch.path().join(&name);
        let memory = Memory::default();
        memory.create_directories(scratch.path())?;
        Disk.write(&path, bytes.clone())?;
        memory.write(&path, bytes.clone())?;
        prop_assert_eq!(&Disk.read(&path)?, &bytes);
        prop_assert_eq!(memory.read(&path)?, bytes);
        prop_assert_eq!(Disk.kind(&path), memory.kind(&path));
    }

    #[test]
    fn disk_and_memory_append_after_what_was_there(name in "[a-z]{1,8}", first in any::<Bytes>(), second in any::<Bytes>()) {
        let scratch = tempfile::tempdir()?;
        let path = scratch.path().join(&name);
        let memory = Memory::default();
        memory.create_directories(scratch.path())?;
        let joined = Bytes::new(first.clone().into_inner().into_iter().chain(second.clone().into_inner()).collect());
        Disk.append(&path, &first).and_then(|()| Disk.append(&path, &second))?;
        memory.append(&path, &first).and_then(|()| memory.append(&path, &second))?;
        prop_assert_eq!(&Disk.read(&path)?, &joined);
        prop_assert_eq!(memory.read(&path)?, joined);
    }

    #[test]
    fn disk_and_memory_copy_and_list_alike(name in "[a-z]{1,8}", copy in "[A-Z]{1,8}", folder in "[0-9]{1,4}", bytes in any::<Bytes>()) {
        let scratch = tempfile::tempdir()?;
        let inner = scratch.path().join(&folder);
        let [from, to] = [scratch.path().join(&name), inner.join(&copy)];
        let memory = Memory::default();
        memory.create_directories(&inner)?;
        Disk.create_directories(&inner)?;
        Disk.write(&from, bytes.clone()).and_then(|()| Disk.copy(&from, &to))?;
        memory.write(&from, bytes.clone()).and_then(|()| memory.copy(&from, &to))?;
        prop_assert_eq!(&Disk.read(&to)?, &bytes);
        prop_assert_eq!(memory.read(&to)?, bytes);
        let mut on_disk = Disk.entries(scratch.path())?;
        let mut in_memory = memory.entries(scratch.path())?;
        on_disk.sort();
        in_memory.sort();
        prop_assert_eq!(on_disk, in_memory);
    }

    #[test]
    fn disk_and_memory_miss_the_same_paths(name in "[a-z]{1,8}") {
        let scratch = tempfile::tempdir()?;
        let path = scratch.path().join(&name);
        let memory = Memory::default();
        memory.create_directories(scratch.path())?;
        prop_assert_eq!(Disk.kind(&path), Kind::Missing);
        prop_assert_eq!(memory.kind(&path), Kind::Missing);
        prop_assert_eq!(Disk.read(&path).is_err(), memory.read(&path).is_err());
        prop_assert_eq!(Disk.size(&path).is_err(), memory.size(&path).is_err());
    }

    #[test]
    fn reading_from_an_offset_skips_that_many_bytes(name in "[a-z]{1,8}", bytes in any::<Bytes>(), offset in any::<Offset>()) {
        let scratch = tempfile::tempdir()?;
        let path = scratch.path().join(&name);
        let memory = Memory::default();
        memory.create_directories(scratch.path())?;
        Disk.write(&path, bytes.clone())?;
        memory.write(&path, bytes.clone())?;
        let skipped = usize::try_from(offset.position()).into_iter().next().unwrap_or(usize::MAX);
        let tail = Bytes::new(bytes.into_inner().into_iter().skip(skipped).collect());
        prop_assert_eq!(&Disk.read_from(&path, offset)?, &tail);
        prop_assert_eq!(memory.read_from(&path, offset)?, tail);
    }

    #[test]
    fn a_cache_reads_and_writes_through_to_its_files(name in "[a-z]{1,8}", first in any::<Bytes>(), second in any::<Bytes>()) {
        let memory = Memory::default();
        let path = PathBuf::from(&name);
        memory.write(&path, first.clone())?;
        let cached = Cached::new(&memory);
        prop_assert_eq!(cached.read(&path)?, first);
        cached.write(&path, second.clone())?;
        prop_assert_eq!(&cached.read(&path)?, &second);
        prop_assert_eq!(memory.read(&path)?, second);
    }

    #[test]
    fn absent_holds_nothing_and_takes_nothing(name in "[a-z]{1,8}", bytes in any::<Bytes>()) {
        let path = PathBuf::from(&name);
        prop_assert_eq!(Absent.write(&path, bytes.clone()).is_err(), Absent.read(&path).is_err());
        prop_assert_eq!(Absent.append(&path, &bytes).is_err(), Absent.entries(&path).is_err());
        prop_assert_eq!(Absent.kind(&path), Kind::Missing);
    }
}
