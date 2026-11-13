use std::mem;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::thread;

use graph::Model;
use graph_reader::{FileFacts, GroupLowering, ListedGroup, Listing};
use lint_core::{FileHead, Parsed, Readout, SourceFile};
use lint_sweep::Gathers;

use super::contexts::{Contexts, Contributed, PrimedParts, Primings};

use super::super::boundary::FileOrder;

#[derive(Debug)]
struct Claim<'listing, 'read, 'run> {
    order: FileOrder,
    group: &'listing ListedGroup<'read>,
    file: &'read SourceFile,
    head: &'run FileHead,
}

impl<'run> Claim<'_, '_, 'run> {
    fn gathered(&self) -> Gathered<'run> {
        let parsed = Parsed::new(self.file, self.head);
        let mut lowering: GroupLowering<'_> = self.group.lowering();
        let contexts = Contributed::joined(parsed, lowering.watching());
        Gathered { head: self.head, facts: lowering.gathered(), contexts }
    }
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

impl<'read> Claimed<'read> {
    pub(crate) fn order(self) -> FileOrder {
        self.order
    }

    pub(crate) fn file(self) -> &'read SourceFile {
        self.file
    }
}

#[derive(Debug)]
pub(crate) struct Share<'read, 'run> {
    claimed: Vec<Claimed<'read>>,
    merging: Option<Merging<'run>>,
}

impl<'read, 'run> Share<'read, 'run> {
    pub(crate) fn claimed(&self) -> &[Claimed<'read>] {
        &self.claimed
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

#[derive(Debug)]
struct PrimedFile<'run> {
    head: &'run FileHead,
    parts: PrimedParts<'run>,
}

#[derive(Debug)]
pub(crate) struct Gathering<'run> {
    heads: &'run OnceLock<Vec<FileHead>>,
    deposited: Mutex<Vec<Gathered<'run>>>,
    primed: Mutex<Vec<PrimedFile<'run>>>,
    primings: OnceLock<Primings<'run>>,
    model: OnceLock<Option<Model>>,
    contexts: OnceLock<Contexts<'run>>,
}

impl<'run> Gathering<'run> {
    pub(crate) fn new(heads: &'run OnceLock<Vec<FileHead>>) -> Gathering<'run> {
        Gathering {
            heads,
            deposited: Mutex::default(),
            primed: Mutex::default(),
            primings: OnceLock::new(),
            model: OnceLock::new(),
            contexts: OnceLock::new(),
        }
    }

    pub(crate) fn gathered<'read>(&self, readout: &'read Readout, table: &[FileHead]) -> Share<'read, 'run> {
        let heads = self.heads.get_or_init(|| table.to_vec());
        let listing = Listing::of_files(readout.files());
        let claims: Vec<Claim<'_, 'read, 'run>> = listing
            .groups()
            .iter()
            .zip(readout.files())
            .filter_map(|(group, file)| {
                let index = heads.binary_search_by(|head| head.path().cmp(file.head().path())).ok()?;
                Some(Claim { order: FileOrder::from(index), group, file, head: heads.get(index)? })
            })
            .collect();
        let (claimed, parts): (Vec<Claimed<'read>>, Vec<Gathered<'run>>) =
            claims.iter().map(|claim| (Claimed { order: claim.order, file: claim.file }, claim.gathered())).unzip();
        Share { claimed, merging: self.deposited(parts, heads) }
    }

    pub(crate) fn settled(&self, merging: Merging<'run>, model: impl FnOnce(Vec<FileFacts>) -> Option<Model>) {
        let split = Self::split(merging);
        let contexts = split.contexts;
        thread::scope(|scope| {
            scope.spawn(|| self.parts_settled(contexts));
            self.model.get_or_init(|| model(split.facts));
        });
    }

    fn parts_settled(&self, contexts: Vec<Contributed<'run>>) {
        self.contexts.get_or_init(|| contexts.into_iter().collect());
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

    pub(crate) fn primed(&self, claimed: &[Claimed<'_>]) -> Option<&Primings<'run>> {
        let contexts = self.contexts.wait();
        let heads = self.heads.get().map_or(&[][..], Vec::as_slice);
        let parts: Vec<PrimedFile<'run>> = claimed
            .iter()
            .filter_map(|found| {
                found
                    .order
                    .picked_from(heads)
                    .map(|head| PrimedFile { head, parts: contexts.priming(Parsed::new(found.file, head)) })
            })
            .collect();
        let mut held = self.primed.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(parts);
        let complete = Some(held.len()).filter(|count| *count == heads.len()).map(|_| mem::take(&mut *held));
        complete.map(Self::primings_of).map(|merged| self.primings.get_or_init(|| merged))
    }

    fn primings_of(files: Vec<PrimedFile<'run>>) -> Primings<'run> {
        let mut sorted = files;
        sorted.sort_by(|earlier, later| earlier.head.path().cmp(later.head.path()));
        sorted.into_iter().map(|file| file.parts).collect()
    }

    pub(crate) fn primings(&self) -> &Primings<'run> {
        self.primings.wait()
    }

    fn deposited(&self, parts: Vec<Gathered<'run>>, heads: &[FileHead]) -> Option<Merging<'run>> {
        let mut held = self.deposited.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(parts);
        let complete = Some(held.len()).filter(|count| *count == heads.len()).map(|_| mem::take(&mut *held));
        complete.map(Merging)
    }

    fn split(merging: Merging<'run>) -> Splitting<'run> {
        let mut sorted = merging.0;
        sorted.sort_by(|earlier, later| earlier.head.path().cmp(later.head.path()));
        sorted.into_iter().fold(Splitting::default(), |mut split, part| {
            split.facts.extend(part.facts);
            split.contexts.push(part.contexts);
            split
        })
    }
}
