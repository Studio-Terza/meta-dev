use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::path::Path;

use code_model::{Building, FileId, Reach};
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::Role;

use crate::lowering::{Lowered, Slot};
use crate::namespace::CrateForm;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Listed<'read> {
    source: &'read SourceFile,
    reach: Reach,
}

impl<'read> From<&'read SourceFile> for Listed<'read> {
    fn from(source: &'read SourceFile) -> Self {
        Listed { source, reach: Listed::reach_of(source) }
    }
}

impl<'read> Listed<'read> {
    pub(crate) fn tested(self) -> Self {
        Listed { source: self.source, reach: Reach::Test }
    }

    fn reach_of(source: &SourceFile) -> Reach {
        match source.shipped() {
            Some(_) => Reach::Shipped,
            None => Reach::Test,
        }
    }

    pub(crate) fn read(self, building: &mut Building) -> ReadFile<'read> {
        ReadFile { source: self.source, file: building.file(self.source.path().to_path_buf()), reach: self.reach }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadFile<'read> {
    source: &'read SourceFile,
    file: FileId,
    reach: Reach,
}

impl<'read> ReadFile<'read> {
    pub(crate) fn reach(self) -> Reach {
        self.reach
    }

    pub(crate) fn source(self) -> &'read SourceFile {
        self.source
    }

    pub(crate) fn file(self) -> FileId {
        self.file
    }

    fn named(self) -> Option<&'read Name> {
        match self.source.placement().role() {
            Role::LibraryRoot => Some(Borrow::<Name>::borrow(self.source.placement().package().crate_name())),
            Role::BinaryRoot | Role::Target | Role::BuildScript | Role::Other => None,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Files<'read>(BTreeMap<&'read Path, ReadFile<'read>>);

impl<'read> Files<'read> {
    pub(crate) fn present(&self, path: &Path) -> Option<ReadFile<'read>> {
        self.0.get(path).copied()
    }

    fn filled(self, lowered: &mut Lowered<'read>, slots: Vec<Slot>) -> Self {
        slots.into_iter().fold(self, |mut files, slot| {
            files.slot_filled(lowered, slot);
            files
        })
    }

    fn rooted(self, lowered: &mut Lowered<'read>, root: ReadFile<'read>, name: Option<&'read Name>) -> Self {
        let slots = lowered.rooted(root, name, CrateForm::Planted);
        self.filled(lowered, slots)
    }

    fn slot_filled(&mut self, lowered: &mut Lowered<'read>, slot: Slot) {
        match slot.found(self) {
            Some(path) => self.planted_at(lowered, slot, &path),
            None => lowered.opaque(slot),
        }
    }

    fn planted_at(&mut self, lowered: &mut Lowered<'read>, slot: Slot, path: &Path) {
        let child = self.0.remove(path);
        let slots = child.map(|child| lowered.planted(slot, child)).unwrap_or_default();
        *self = std::mem::take(self).filled(lowered, slots);
    }
}

#[derive(Debug)]
struct CrateFiles<'read> {
    root: Option<ReadFile<'read>>,
    files: Files<'read>,
    name: Option<&'read Name>,
}

impl<'read> CrateFiles<'read> {
    fn new(crate_file: &Path, read: Vec<ReadFile<'read>>) -> Self {
        let mut files: BTreeMap<&'read Path, ReadFile<'read>> =
            read.into_iter().map(|file| (file.source.path(), file)).collect();
        let root = files.remove(crate_file);
        CrateFiles { name: root.and_then(ReadFile::named), root, files: Files(files) }
    }

    fn planted(self, lowered: &mut Lowered<'read>) {
        self.remaining(lowered).0.into_values().for_each(|island| lowered.island(island));
    }

    fn remaining(self, lowered: &mut Lowered<'read>) -> Files<'read> {
        match self.root {
            Some(root) => self.files.rooted(lowered, root, self.name),
            None => self.files,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Planting<'read> {
    crates: Vec<CrateFiles<'read>>,
}

impl<'read> Planting<'read> {
    pub(crate) fn new(read: &[ReadFile<'read>]) -> Self {
        let grouped =
            read.iter().fold(BTreeMap::new(), |mut grouped: BTreeMap<&'read Path, Vec<ReadFile<'read>>>, file| {
                grouped.entry(file.source.crate_file().as_ref()).or_default().push(*file);
                grouped
            });
        Planting { crates: grouped.into_iter().map(|(crate_file, files)| CrateFiles::new(crate_file, files)).collect() }
    }

    pub(crate) fn lowered(self) -> Lowered<'read> {
        self.crates.into_iter().fold(Lowered::default(), |mut lowered, krate| {
            krate.planted(&mut lowered);
            lowered
        })
    }
}
