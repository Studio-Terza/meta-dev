mod manifest_file;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lint_placement::{CrateFile, Placement, Role};

use super::located::Located;
use manifest_file::ManifestFile;

#[derive(Clone, Debug)]
pub struct FileHead {
    path: PathBuf,
    located: Located,
    placement: Placement,
    crate_file: CrateFile,
    manifest: OnceLock<ManifestFile>,
}

impl FileHead {
    const MANIFEST: &'static str = "Cargo.toml";

    pub(super) fn new(path: PathBuf, located: Located, placement: Placement) -> FileHead {
        FileHead { crate_file: placement.crate_file(&path), path, located, placement, manifest: OnceLock::new() }
    }

    pub(super) fn in_crate_file(self, crate_file: CrateFile) -> FileHead {
        FileHead { crate_file, ..self }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
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
    pub fn crate_file(&self) -> &CrateFile {
        &self.crate_file
    }

    #[must_use]
    pub fn shipped(&self) -> Option<&FileHead> {
        (self.placement.role() != Role::Target).then_some(self)
    }

    #[must_use]
    pub fn switched_off(&self) -> Option<&Path> {
        self.placement.package().untested().holds(self.located.within(&self.path)).map(|_| self.path.as_path())
    }

    #[must_use]
    pub fn manifest_file(&self) -> &ManifestFile {
        self.manifest.get_or_init(|| {
            let inside = self.located.within(&self.path).strip_prefix(self.placement.package().root().directory()).ok();
            let depth = inside.map_or(0, |relative| relative.components().count());
            ManifestFile::from(self.path.ancestors().nth(depth).unwrap_or(&self.path).join(Self::MANIFEST))
        })
    }
}
