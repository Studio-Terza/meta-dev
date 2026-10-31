use std::fs;
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static MADE: AtomicU64 = AtomicU64::new(0);

/// A fresh, empty directory under the system temp directory, for tests. It is removed with
/// everything in it when dropped, so a test that makes one leaves nothing behind, even when it
/// fails. It reads as the `Path` of the directory.
#[derive(Debug)]
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_nanos());
        let serial = MADE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("meta-dev-{name}-{}-{nanos}-{serial}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// Lets the owner write every directory under `dir`, which a read-only tree does not, so that it can
/// be removed. Follows no link.
fn thaw(dir: &Path) {
    let opened = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
    drop(opened);
    let entries = fs::read_dir(dir).into_iter().flatten().flatten();
    entries.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir())).for_each(|entry| thaw(&entry.path()));
}

impl Drop for Scratch {
    /// A directory that cannot be removed is only left behind; it must not fail the test.
    fn drop(&mut self) {
        thaw(&self.0);
        let removed = fs::remove_dir_all(&self.0);
        drop(removed);
    }
}

#[cfg(test)]
mod tests {
    use super::Scratch;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    #[test]
    fn dropping_the_scratch_removes_its_directory_and_everything_in_it() {
        let scratch = Scratch::new("guard");
        fs::create_dir_all(scratch.join("deep/er")).unwrap();
        fs::write(scratch.join("deep/er/file"), "x").unwrap();
        let kept: PathBuf = scratch.to_path_buf();
        assert!(kept.is_dir());
        drop(scratch);
        assert!(!kept.exists());
    }

    #[test]
    fn dropping_the_scratch_removes_a_read_only_tree_too() {
        let scratch = Scratch::new("guard-frozen");
        fs::create_dir_all(scratch.join("deep/er")).unwrap();
        fs::write(scratch.join("deep/er/file"), "x").unwrap();
        for path in ["deep/er", "deep"] {
            fs::set_permissions(scratch.join(path), fs::Permissions::from_mode(0o500)).unwrap();
        }
        let kept: PathBuf = scratch.to_path_buf();
        drop(scratch);
        assert!(!kept.exists());
    }

    #[test]
    fn two_scratches_with_the_same_name_are_different_directories() {
        let (first, second) = (Scratch::new("same"), Scratch::new("same"));
        assert_ne!(*first, *second);
    }
}
