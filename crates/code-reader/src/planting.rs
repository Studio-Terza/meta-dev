use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use code_model::{Building, FileId, Model};
use lint_core::SourceFile;
use lint_layout::{Dependencies, Name};
use lint_placement::{Placement, Role};

use crate::lowering::{Lowered, Plot, Slot};
use crate::namespace::{CrateForm, Top};

#[derive(Debug)]
pub(crate) struct Placing {
    path: PathBuf,
    crate_file: PathBuf,
    name: Option<Name>,
    krate: Name,
    dependencies: Dependencies,
}

impl From<&SourceFile> for Placing {
    fn from(source: &SourceFile) -> Placing {
        let placement = source.placement();
        Placing {
            path: source.path().to_path_buf(),
            crate_file: AsRef::<Path>::as_ref(source.crate_file()).to_path_buf(),
            name: Self::library(placement),
            krate: Borrow::<Name>::borrow(placement.package().crate_name()).clone(),
            dependencies: placement.package().dependencies().clone(),
        }
    }
}

impl Placing {
    fn library(placement: &Placement) -> Option<Name> {
        match placement.role() {
            Role::LibraryRoot => Some(Borrow::<Name>::borrow(placement.package().crate_name()).clone()),
            Role::BinaryRoot | Role::Target | Role::BuildScript | Role::Other => None,
        }
    }

    fn top(&self, form: CrateForm) -> Top<'_> {
        Top::Root { form, name: self.name.as_ref(), dependencies: &self.dependencies }
    }

    fn island(&self) -> Top<'_> {
        Top::Root { form: CrateForm::Island, name: None, dependencies: &self.dependencies }
    }
}

#[derive(Debug)]
struct Unplanted<'read> {
    placing: &'read Placing,
    file: FileId,
    lowered: Lowered,
}

#[derive(Debug, Default)]
pub(crate) struct Files<'read>(BTreeMap<&'read Path, Unplanted<'read>>);

impl<'read> Files<'read> {
    pub(crate) fn present(&self, path: &Path) -> Option<FileId> {
        self.0.get(path).map(|unplanted| unplanted.file)
    }

    fn filled(self, planted: &mut Plot<'read>, slots: Vec<Slot>) -> Files<'read> {
        slots.into_iter().fold(self, |mut files, slot| {
            files.slot_filled(planted, slot);
            files
        })
    }

    fn rooted(self, planted: &mut Plot<'read>, root: Unplanted<'read>) -> Files<'read> {
        let slots = planted.rooted(root.placing.top(CrateForm::Planted), root.file, root.lowered);
        self.filled(planted, slots)
    }

    fn slot_filled(&mut self, planted: &mut Plot<'read>, slot: Slot) {
        match slot.found(self) {
            Some(path) => self.planted_at(planted, slot, &path),
            None => planted.opaque(slot),
        }
    }

    fn planted_at(&mut self, planted: &mut Plot<'read>, slot: Slot, path: &Path) {
        let child = self.0.remove(path);
        let slots = child.map_or_default(|removed| planted.planted(slot, removed.file, removed.lowered));
        *self = std::mem::take(self).filled(planted, slots);
    }
}

#[derive(Debug)]
struct CrateFiles<'read> {
    root: Option<Unplanted<'read>>,
    files: Files<'read>,
}

impl<'read> CrateFiles<'read> {
    fn new(crate_file: &Path, unplanted: Vec<Unplanted<'read>>) -> CrateFiles<'read> {
        let mut files: BTreeMap<&'read Path, Unplanted<'read>> =
            unplanted.into_iter().map(|file| (file.placing.path.as_path(), file)).collect();
        let root = files.remove(crate_file);
        CrateFiles { root, files: Files(files) }
    }

    fn planted(self, planted: &mut Plot<'read>) {
        self.remaining(planted).0.into_values().for_each(|island| {
            let slots = planted.rooted(island.placing.island(), island.file, island.lowered);
            slots.into_iter().fold(&mut *planted, |opaqued, slot| {
                opaqued.opaque(slot);
                opaqued
            });
        });
    }

    fn remaining(self, planted: &mut Plot<'read>) -> Files<'read> {
        match self.root {
            Some(root) => self.files.rooted(planted, root),
            None => self.files,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Planting<'read> {
    crates: Vec<CrateFiles<'read>>,
}

impl<'read> Planting<'read> {
    pub(crate) fn merged(placings: &'read [Placing], bodies: Vec<Lowered>) -> Model {
        let mut building = Building::default();
        let unplanted = placings
            .iter()
            .zip(bodies)
            .map(|(placing, lowered)| Unplanted {
                placing,
                file: building.file(placing.path.clone(), placing.krate.clone()),
                lowered,
            })
            .collect();
        Planting::new(unplanted).planted().built(building)
    }

    fn new(unplanted: Vec<Unplanted<'read>>) -> Planting<'read> {
        let grouped = unplanted.into_iter().fold(
            BTreeMap::new(),
            |mut grouped: BTreeMap<&'read Path, Vec<Unplanted<'read>>>, file| {
                grouped.entry(file.placing.crate_file.as_path()).or_default().push(file);
                grouped
            },
        );
        Planting { crates: grouped.into_iter().map(|(crate_file, files)| CrateFiles::new(crate_file, files)).collect() }
    }

    fn planted(self) -> Plot<'read> {
        self.crates.into_iter().fold(Plot::default(), |mut planted, krate| {
            krate.planted(&mut planted);
            planted
        })
    }
}
