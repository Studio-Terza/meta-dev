mod manifest_file;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lint_placement::{CrateFile, Placement, Role};

use super::located::Located;
use crate::ignored::Ignored;
use invariants::Ignoring;
use manifest_file::ManifestFile;

#[derive(Clone, Debug)]
pub struct FileHead {
    path: PathBuf,
    located: Located,
    placement: Placement,
    crate_file: CrateFile,
    manifest: OnceLock<ManifestFile>,
    tests: Vec<FileHead>,
}

impl FileHead {
    const MANIFEST: &'static str = "Cargo.toml";
    const SOURCE: &'static str = "src";

    pub(super) fn new(path: PathBuf, located: Located, placement: Placement) -> FileHead {
        FileHead {
            crate_file: placement.crate_file(&path),
            path,
            located,
            placement,
            manifest: OnceLock::new(),
            tests: Vec::new(),
        }
    }

    pub(super) fn in_crate_file(self, crate_file: CrateFile) -> FileHead {
        FileHead { crate_file, ..self }
    }

    pub(super) fn with_tests(self, tests: Vec<FileHead>) -> FileHead {
        FileHead { tests, ..self }
    }

    #[must_use]
    pub fn test_heads(&self) -> &[FileHead] {
        &self.tests
    }

    #[must_use]
    pub fn located(&self) -> &Located {
        &self.located
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    #[must_use]
    pub fn shipped(&self) -> Option<Role> {
        Some(self.placement.role()).filter(|role| *role != Role::Target)
    }

    #[must_use]
    pub fn binary(&self) -> Option<&CrateFile> {
        let source = self.placement.crate_root().directory().join(Self::SOURCE);
        let inside = self.crate_file.as_ref().strip_prefix(source).ignoring(Ignored::Outside);
        inside.map(Role::in_source).filter(|role| *role == Role::BinaryRoot).map(|_| &self.crate_file)
    }

    #[must_use]
    pub fn manifest_file(&self) -> &ManifestFile {
        self.manifest.get_or_init(|| {
            let inside = self
                .located
                .within(&self.path)
                .strip_prefix(self.placement.package().root().directory())
                .ignoring(Ignored::Outside);
            let depth = inside.map_or(0, |relative| relative.components().count());
            ManifestFile::from(self.path.ancestors().nth(depth).unwrap_or(&self.path).join(Self::MANIFEST))
        })
    }
}

impl AsRef<Path> for FileHead {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
