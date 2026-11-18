use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use algebra::{Monoid, Semigroup};
use graph::{Building, FileHome, FileId, Harness, Model};
use lint_core::{SourceFile, TestFile};
use lint_layout::{Dependencies, Name};
use lint_placement::{Placement, Role};

use crate::lowering::{Lowered, Plot, Plotting, SlotAt};
use crate::namespace::{CrateForm, Rooted, Top};

#[derive(Debug)]
pub(crate) struct Placing {
    path: PathBuf,
    crate_file: PathBuf,
    name: Option<Name>,
    krate: Name,
    module: Vec<Name>,
    dependencies: Dependencies,
    harness: Harness,
}

impl From<&SourceFile> for Placing {
    fn from(source: &SourceFile) -> Placing {
        let placement = source.head().placement();
        Placing {
            path: AsRef::<Path>::as_ref(source.head()).to_path_buf(),
            crate_file: AsRef::<Path>::as_ref(source.head().crate_file()).to_path_buf(),
            name: Self::library(placement),
            krate: Borrow::<Name>::borrow(placement.package().crate_name()).clone(),
            module: AsRef::<[Name]>::as_ref(placement.module()).to_vec(),
            dependencies: placement.package().dependencies().clone(),
            harness: TestFile::tested(source).map_or(Harness::Elsewhere, |_| Harness::TestTarget),
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
        Top::Root(Rooted::new(form, self.name.as_ref(), &self.dependencies))
    }

    fn island(&self) -> Top<'_> {
        Top::Root(Rooted::new(CrateForm::Island, None, &self.dependencies))
    }
}

#[derive(Debug)]
struct Unplanted<'read> {
    placing: &'read Placing,
    file: FileId,
    lowered: Lowered,
}

#[derive(Debug, Default)]
struct Files<'read>(BTreeMap<&'read Path, Unplanted<'read>>);

#[derive(Clone, Copy, Debug)]
struct Claims<'up, 'read> {
    claimed: &'up BTreeSet<&'read Path>,
    up: Option<&'up Claims<'up, 'read>>,
}

impl Claims<'_, '_> {
    fn holds(self, path: &Path) -> Option<()> {
        self.claimed.get(path).map(|_| ()).or_else(|| self.up.and_then(|up| up.holds(path)))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Unclaimed<'files, 'up, 'read> {
    files: &'files Files<'read>,
    claims: Claims<'up, 'read>,
}

impl Unclaimed<'_, '_, '_> {
    pub(crate) fn present(self, path: &Path) -> Option<FileId> {
        self.claims.holds(path).map_or_else(|| self.files.0.get(path).map(|unplanted| unplanted.file), |()| None)
    }
}

#[derive(Clone, Copy, Debug)]
enum Sprout {
    Root(FileId),
    Island(FileId),
    Child { at: SlotAt, file: FileId },
    Opaque(SlotAt),
}

impl Sprout {
    fn file(self) -> Option<FileId> {
        match self {
            Sprout::Root(file) | Sprout::Island(file) | Sprout::Child { file, .. } => Some(file),
            Sprout::Opaque(_) => None,
        }
    }
}

#[derive(Debug, Default)]
struct Sprouts<'read> {
    order: Vec<Sprout>,
    claimed: BTreeSet<&'read Path>,
}

impl Semigroup for Sprouts<'_> {
    fn combine(self, other: Self) -> Self {
        let mut claimed = self.claimed;
        claimed.extend(other.claimed);
        Sprouts { order: self.order.combine(other.order), claimed }
    }
}

impl Monoid for Sprouts<'_> {}

impl From<Sprout> for Sprouts<'_> {
    fn from(sprout: Sprout) -> Self {
        Sprouts { order: vec![sprout], claimed: BTreeSet::new() }
    }
}

impl<'read> Sprouts<'read> {
    fn island(unplanted: &Unplanted<'read>) -> Sprouts<'read> {
        let opaque = SlotAt::every(unplanted.file, &unplanted.lowered).map(|at| Sprouts::from(Sprout::Opaque(at)));
        Sprouts::from(Sprout::Island(unplanted.file)).combine(Monoid::concat(opaque))
    }
}

impl<'read> Files<'read> {
    fn branches(&self, parent: &Unplanted<'read>, claims: Claims<'_, 'read>) -> Sprouts<'read> {
        let slots = SlotAt::every(parent.file, &parent.lowered).zip(parent.lowered.slots());
        slots.fold(Sprouts::default(), |sprouts, (at, slot)| {
            let here = Claims { claimed: &sprouts.claimed, up: Some(&claims) };
            let found = slot.found(Unclaimed { files: self, claims: here });
            let next = self.sprouted(at, found.as_deref(), here);
            sprouts.combine(next)
        })
    }

    fn sprouted(&self, at: SlotAt, found: Option<&Path>, claims: Claims<'_, 'read>) -> Sprouts<'read> {
        let child = found.and_then(|path| self.0.get_key_value(path));
        child.map_or_else(
            || Sprouts::from(Sprout::Opaque(at)),
            |(path, unplanted)| {
                let claimed = BTreeSet::from([*path]);
                let below = self.branches(unplanted, Claims { claimed: &claimed, up: Some(&claims) });
                let own = Sprouts { order: vec![Sprout::Child { at, file: unplanted.file }], claimed };
                own.combine(below)
            },
        )
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

    fn sprouts(&self) -> Sprouts<'read> {
        let none = BTreeSet::new();
        let rooted = self.root.as_ref().map_or_default(|root| {
            let below = self.files.branches(root, Claims { claimed: &none, up: None });
            Sprouts::from(Sprout::Root(root.file)).combine(below)
        });
        let claims = Claims { claimed: &rooted.claimed, up: None };
        let remaining = self.files.0.iter().filter(|(path, _)| claims.holds(path).is_none());
        let islands = Monoid::concat(remaining.map(|(_, island)| Sprouts::island(island)));
        rooted.combine(islands)
    }

    fn unplanted(self) -> impl Iterator<Item = Unplanted<'read>> {
        self.root.into_iter().chain(self.files.0.into_values())
    }
}

#[derive(Debug)]
struct Sprouting<'read> {
    sprout: Sprout,
    unplanted: Option<Unplanted<'read>>,
}

impl<'read> Sprouting<'read> {
    fn grown(self, plotting: Plotting<'_, 'read>) -> Plot<'read> {
        match self.sprout {
            Sprout::Root(_) => self
                .unplanted
                .map_or_default(|root| plotting.rooted(root.placing.top(CrateForm::Planted), root.file, root.lowered)),
            Sprout::Island(_) => self
                .unplanted
                .map_or_default(|island| plotting.rooted(island.placing.island(), island.file, island.lowered)),
            Sprout::Child { at, .. } => {
                self.unplanted.map_or_default(|child| plotting.planted(at, child.file, child.lowered))
            }
            Sprout::Opaque(at) => plotting.opaque(at),
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
                file: building.file(
                    placing.path.clone(),
                    FileHome::new(
                        placing.krate.clone(),
                        placing.crate_file.clone(),
                        placing.module.clone(),
                        placing.harness,
                    ),
                ),
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
        let sprouts: Sprouts<'read> = Monoid::concat(self.crates.iter().map(CrateFiles::sprouts));
        let owned: BTreeMap<FileId, Unplanted<'read>> =
            self.crates.into_iter().flat_map(CrateFiles::unplanted).map(|file| (file.file, file)).collect();
        let sproutings = sprouts.order.into_iter().scan(owned, |waiting, sprout| {
            Some(Sprouting { sprout, unplanted: sprout.file().and_then(|file| waiting.remove(&file)) })
        });
        sproutings.fold(Plot::default(), |plot, sprouting| {
            let grown = sprouting.grown(Plotting::from(&plot));
            plot.combine(grown)
        })
    }
}
