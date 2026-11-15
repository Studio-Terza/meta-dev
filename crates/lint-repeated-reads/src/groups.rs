mod dispatched;
mod finding;
mod read_key;

use std::collections::{BTreeMap, BTreeSet};

use graph::{Function, Model, Placed, Resolved, Seat, TypeRef};
use lint_core::{Site, Violation};

use dispatched::Dispatched;
use finding::Finding;
use read_key::ReadKey;

#[derive(Debug)]
struct Keyed<'model> {
    key: ReadKey<'model>,
    subject: &'model Resolved,
    site: Site<'model>,
}

#[derive(Debug, Default)]
struct Readers<'model> {
    subjects: BTreeSet<&'model Resolved>,
    sites: Vec<Site<'model>>,
}

#[derive(Debug)]
pub struct Groups<'model> {
    model: &'model Model,
    grouped: BTreeMap<ReadKey<'model>, Readers<'model>>,
}

impl<'model> Groups<'model> {
    const SHARED_BY: usize = 2;

    pub fn found(model: &'model Model) -> Groups<'model> {
        let dispatched = Dispatched::in_model(model);
        let grouped = model
            .functions()
            .flat_map(Self::keyed)
            .filter(|keyed| keyed.key.run_together(&dispatched).is_some())
            .fold(BTreeMap::<ReadKey<'_>, Readers<'_>>::new(), |mut groups, keyed| {
                let readers = groups.entry(keyed.key).or_default();
                readers.subjects.insert(keyed.subject);
                readers.sites.push(keyed.site);
                groups
            });
        Groups { model, grouped }
    }

    pub fn violations(self) -> Vec<Violation<'model>> {
        let model = self.model;
        self.grouped
            .into_iter()
            .filter(|(_, readers)| readers.subjects.len() >= Self::SHARED_BY)
            .filter_map(|(key, readers)| {
                let first = readers.sites.iter().min().copied()?;
                Some(Violation::new(first, Finding::new(model, key, readers.sites)))
            })
            .collect()
    }

    fn keyed(placed: Placed<'model, &'model Function>) -> Vec<Keyed<'model>> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let subject = seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        let reads = function.definition().map_or(&[][..], graph::Definition::reads);
        let owned = seat.implemented_trait().zip(subject);
        owned.map_or_default(|(implemented, implementer)| {
            reads
                .iter()
                .filter_map(|read| ReadKey::of_read(function, implemented, read))
                .map(|key| Keyed { key, subject: implementer, site: placed.site() })
                .collect()
        })
    }
}
