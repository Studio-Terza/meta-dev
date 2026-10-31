use std::fs;
use std::ops::Deref;
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

impl Drop for Scratch {
    /// A directory that cannot be removed is only left behind; it must not fail the test.
    fn drop(&mut self) {
        let removed = fs::remove_dir_all(&self.0);
        drop(removed);
    }
}

#[cfg(test)]
mod tests {
    use super::Scratch;
    use std::fs;
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
    fn two_scratches_with_the_same_name_are_different_directories() {
        let (first, second) = (Scratch::new("same"), Scratch::new("same"));
        assert_ne!(*first, *second);
    }
}
