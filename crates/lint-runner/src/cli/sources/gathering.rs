use std::mem;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::thread;

use graph::Model;
use graph_reader::{FileFacts, GroupLowering, ListedGroup, Listing};
use lint_core::{FileHead, Parsed, Readout, SourceFile, Unwalked};
use lint_sweep::Gathers;

use super::super::boundary::{Filings, Kept, Store, Worked};
use super::super::timings::{Measured, Timing};
use super::contexts::{Contexts, Contributed};
use crate::cli::ignored::Ignored;
use invariants::{Ignoring, Rejoined, Unpoisoned};

use super::super::boundary::FileOrder;

#[derive(Debug)]
struct Claim<'listing, 'read, 'run> {
    order: FileOrder,
    group: &'listing ListedGroup<'read>,
    file: &'read SourceFile,
    head: &'run FileHead,
    unwalked: Unwalked,
}

impl<'read, 'run> Claim<'_, 'read, 'run> {
    fn gathered<Part: AsRef<Worked>, Filed: Gathers<Part>>(
        self,
        store: &Store,
        starting: &impl Fn(Claimed<'read>, Option<Filings>) -> Filed,
    ) -> Joined<'run, Part> {
        let key = store.key(self.file);
        let recalled = key.as_ref().and_then(|found| store.recalled(found));
        let parsed = Parsed::new(self.file, self.head);
        let lowering: GroupLowering<'_> = self.group.lowering();
        let filing = starting(Claimed { order: self.order, file: self.file }, recalled);
        let contexts = Contributed::joined(parsed, self.unwalked);
        let gathered = Gathered { head: self.head, facts: lowering.gathered(self.file), contexts };
        let filed = filing.gathered(self.file);
        let kept = key.map(|found| Kept::remembered(found, filed.as_ref()));
        Joined { gathered, filed, kept }
    }
}

#[derive(Debug)]
struct Joined<'run, Part> {
    gathered: Gathered<'run>,
    filed: Part,
    kept: Option<Kept>,
}

#[derive(Debug)]
struct Gathered<'run> {
    head: &'run FileHead,
    facts: Vec<FileFacts>,
    contexts: Contributed<'run>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Claimed<'read> {
    order: FileOrder,
    file: &'read SourceFile,
}

impl Claimed<'_> {
    pub(crate) fn order(self) -> FileOrder {
        self.order
    }
}

#[derive(Debug)]
pub(crate) struct Share<'run, Part> {
    merging: Option<Merging<'run>>,
    filed: Vec<Part>,
}

impl<'run, Part> Share<'run, Part> {
    pub(crate) fn filed(&mut self) -> Vec<Part> {
        mem::take(&mut self.filed)
    }

    pub(crate) fn into_merging(self) -> Option<Merging<'run>> {
        self.merging
    }
}

#[derive(Debug, Default)]
struct Splitting<'run> {
    facts: Vec<FileFacts>,
    contexts: Vec<Contributed<'run>>,
}

#[derive(Debug)]
pub(crate) struct Merging<'run>(Vec<Gathered<'run>>);

impl<'run> Merging<'run> {
    pub(crate) fn settled(
        self,
        gathering: &Gathering<'run>,
        model: impl FnOnce(Vec<FileFacts>) -> Measured<Option<Model>>,
        contexts: impl FnOnce(Vec<Contributed<'run>>) -> Measured<Contexts<'run>> + Send,
    ) -> Vec<Timing> {
        let split = self.split();
        let parts = split.contexts;
        thread::scope(|scope| {
            let joined = scope.spawn(|| contexts(parts).kept(&gathering.contexts));
            let modelled = model(split.facts).kept(&gathering.model);
            vec![modelled, joined.join().rejoined()]
        })
    }

    fn split(self) -> Splitting<'run> {
        let mut sorted = self.0;
        sorted.sort_by(|earlier, later| AsRef::<Path>::as_ref(earlier.head).cmp(AsRef::<Path>::as_ref(later.head)));
        sorted.into_iter().fold(Splitting::default(), |mut split, part| {
            split.facts.extend(part.facts);
            split.contexts.push(part.contexts);
            split
        })
    }
}

#[derive(Debug)]
pub(crate) struct Gathering<'run> {
    heads: &'run OnceLock<Vec<FileHead>>,
    deposited: Mutex<Vec<Gathered<'run>>>,
    kept: Mutex<Vec<Kept>>,
    model: OnceLock<Option<Model>>,
    contexts: OnceLock<Contexts<'run>>,
}

impl<'run> Gathering<'run> {
    pub(crate) fn new(heads: &'run OnceLock<Vec<FileHead>>) -> Gathering<'run> {
        Gathering {
            heads,
            deposited: Mutex::default(),
            kept: Mutex::default(),
            model: OnceLock::new(),
            contexts: OnceLock::new(),
        }
    }

    pub(crate) fn gathered<'read, Part: AsRef<Worked>, Filed: Gathers<Part>>(
        &self,
        readout: &'read Readout,
        table: &[FileHead],
        store: &Store,
        starting: impl Fn(Claimed<'read>, Option<Filings>) -> Filed,
    ) -> Share<'run, Part> {
        let heads = self.heads.get_or_init(|| table.to_vec());
        let listing = Listing::of_files(readout.files());
        let claims: Vec<Claim<'_, 'read, 'run>> = listing
            .groups()
            .iter()
            .zip(readout.files())
            .zip(readout.unwalked())
            .filter_map(|((group, file), unwalked)| {
                let index = heads
                    .binary_search_by(|head| AsRef::<Path>::as_ref(head).cmp(AsRef::<Path>::as_ref(file.head())))
                    .ignoring(Ignored::Unclaimed)?;
                Some(Claim { order: FileOrder::from(index), group, file, head: heads.get(index)?, unwalked })
            })
            .collect();
        let joined: Vec<Joined<'run, Part>> =
            claims.into_iter().map(|claim| claim.gathered(store, &starting)).collect();
        let (parts, kept): (Vec<(Gathered<'run>, Part)>, Vec<Option<Kept>>) =
            joined.into_iter().map(|found| ((found.gathered, found.filed), found.kept)).unzip();
        self.kept.lock().unpoisoned().extend(kept.into_iter().flatten());
        let (gathered, filed): (Vec<Gathered<'run>>, Vec<Part>) = parts.into_iter().unzip();
        Share { merging: self.deposited(gathered, heads), filed }
    }

    pub(crate) fn kept(&self) -> Vec<Kept> {
        self.contexts.wait();
        mem::take(&mut *self.kept.lock().unpoisoned())
    }

    pub(crate) fn model(&self) -> Option<&Model> {
        self.model.wait().as_ref()
    }

    pub(crate) fn heads(&self) -> &'run [FileHead] {
        self.heads.get().map_or(&[][..], Vec::as_slice)
    }

    pub(crate) fn contexts(&self) -> &Contexts<'run> {
        self.contexts.wait()
    }

    fn deposited(&self, parts: Vec<Gathered<'run>>, heads: &[FileHead]) -> Option<Merging<'run>> {
        let mut held = self.deposited.lock().unpoisoned();
        held.extend(parts);
        let complete = Some(held.len()).filter(|count| *count == heads.len()).map(|_| mem::take(&mut *held));
        complete.map(Merging)
    }
}

impl<'read> From<Claimed<'read>> for &'read SourceFile {
    fn from(claimed: Claimed<'read>) -> &'read SourceFile {
        claimed.file
    }
}
