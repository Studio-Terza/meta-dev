mod finding;

use std::collections::BTreeSet;

use graph::{Ground, ItemPath, Model, PathUse, Placed, Region, Resolved, Seat, TypeRef};
use lint_core::Violation;
use lint_layout::{PathTail, Word, words};

use finding::Finding;

#[derive(Debug)]
pub struct RawPaths;

#[derive(Debug)]
struct Guards<'model>(BTreeSet<&'model Resolved>);

impl<'model> Guards<'model> {
    const DROP: [Word<'static>; 1] = words!["Drop"];

    fn dropped_in(model: &'model Model) -> Guards<'model> {
        let dropped = model.implementations().filter_map(|placed| {
            let implementation = placed.entity();
            implementation.implemented().and_then(Resolved::std)?.ends_with(PathTail::new(&Self::DROP))?;
            implementation.subject_type().named().map(AsRef::<Resolved>::as_ref)
        });
        Guards(dropped.collect())
    }

    fn covering(&self, used: &PathUse) -> Option<&'model Resolved> {
        let seat: &Seat = used.as_ref();
        let subject = seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        subject.and_then(|named| self.0.get(named)).copied()
    }
}

impl RawPaths {
    const RAW: [[Word<'static>; 2]; 3] =
        [words!["fs", "remove_dir_all"], words!["mem", "forget"], words!["env", "temp_dir"]];

    #[must_use]
    pub fn found(model: &Model) -> Vec<Violation<'_>> {
        let guards = Guards::dropped_in(model);
        model.uses().filter_map(|placed| Self::violation(placed, &guards)).collect()
    }

    fn violation<'model>(placed: Placed<'model, &'model PathUse>, guards: &Guards<'_>) -> Option<Violation<'model>> {
        let used = placed.entity();
        let path = Self::RAW.iter().find_map(|tail| used.spelling(PathTail::new(tail)))?;
        Self::finding(used, path, guards).map(|finding| Violation::new(placed.site(), finding))
    }

    fn finding<'path>(used: &PathUse, path: &'path ItemPath, guards: &Guards<'_>) -> Option<Finding<'path>> {
        match AsRef::<Region>::as_ref(used).ground() {
            Ground::Open | Ground::Store => Some(Finding::OutsidePorts(path)),
            Ground::Ports => Self::unguarded(guards.covering(used), path),
        }
    }

    fn unguarded<'path>(cover: Option<&Resolved>, path: &'path ItemPath) -> Option<Finding<'path>> {
        match cover {
            Some(_) => None,
            None => Some(Finding::OutsideGuard(path)),
        }
    }
}
