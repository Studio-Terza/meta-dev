use std::mem;
use std::sync::{Mutex, OnceLock, PoisonError};

use code_model::Model;
use code_reader::{FileFacts, ListedGroup, Listing};
use lint_core::{FileHead, Parsed, Readout, SourceFile};
use lint_transition_graph::Extraction;

use super::contexts::{Contexts, Contributed};

use super::super::boundary::{FileOrder, Queue};

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
        let extraction = Extraction::from(parsed);
        Gathered { head: self.head, facts: self.group.lowered(), extraction, contexts: Contributed::from(parsed) }
    }
}

#[derive(Debug)]
struct Gathered<'run> {
    head: &'run FileHead,
    facts: Vec<FileFacts>,
    extraction: Extraction<'run>,
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
    extractions: Vec<Extraction<'run>>,
    contexts: Vec<Contributed<'run>>,
}

#[derive(Debug)]
pub(crate) struct Merging<'run> {
    facts: Vec<FileFacts>,
    extraction: Extraction<'run>,
    contexts: Contexts<'run>,
}

#[derive(Debug)]
pub(crate) struct Gathering<'run> {
    heads: &'run OnceLock<Vec<FileHead>>,
    claims: Queue,
    deposited: Mutex<Vec<Gathered<'run>>>,
    model: OnceLock<Option<Model>>,
    extraction: OnceLock<Extraction<'run>>,
    contexts: OnceLock<Contexts<'run>>,
}

impl<'run> Gathering<'run> {
    pub(crate) fn new(heads: &'run OnceLock<Vec<FileHead>>) -> Gathering<'run> {
        Gathering {
            heads,
            claims: Queue::default(),
            deposited: Mutex::default(),
            model: OnceLock::new(),
            extraction: OnceLock::new(),
            contexts: OnceLock::new(),
        }
    }

    pub(crate) fn gathered<'read>(&self, readout: &'read Readout) -> Share<'read, 'run> {
        let heads = self.heads.get_or_init(|| readout.files().iter().map(|file| file.head().clone()).collect());
        let listing = Listing::of_files(readout.files());
        let claims: Vec<Claim<'_, 'read, 'run>> = listing
            .groups()
            .iter()
            .zip(readout.files())
            .zip(heads)
            .enumerate()
            .map(|(index, ((group, file), head))| Claim { order: FileOrder::from(index), group, file, head })
            .collect();
        let (claimed, parts): (Vec<Claimed<'read>>, Vec<Gathered<'run>>) = self
            .claims
            .claimed(&claims)
            .map(|claim| (Claimed { order: claim.order, file: claim.file }, claim.gathered()))
            .unzip();
        Share { claimed, merging: self.deposited(parts, heads) }
    }

    pub(crate) fn settled(&self, merging: Merging<'run>, model: impl FnOnce(Vec<FileFacts>) -> Option<Model>) {
        self.extraction.get_or_init(|| merging.extraction);
        self.contexts.get_or_init(|| merging.contexts);
        self.model.get_or_init(|| model(merging.facts));
    }

    pub(crate) fn model(&self) -> Option<&Model> {
        self.model.wait().as_ref()
    }

    pub(crate) fn contexts(&self) -> &Contexts<'run> {
        self.contexts.wait()
    }

    pub(crate) fn extraction(&self) -> &Extraction<'run> {
        self.extraction.wait()
    }

    fn deposited(&self, parts: Vec<Gathered<'run>>, heads: &[FileHead]) -> Option<Merging<'run>> {
        let mut held = self.deposited.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(parts);
        let complete = Some(held.len()).filter(|count| *count == heads.len()).map(|_| mem::take(&mut *held));
        complete.map(Self::merging)
    }

    fn merging(parts: Vec<Gathered<'run>>) -> Merging<'run> {
        let mut sorted = parts;
        sorted.sort_by(|earlier, later| earlier.head.path().cmp(later.head.path()));
        let split = sorted.into_iter().fold(Splitting::default(), |mut merging, part| {
            merging.facts.extend(part.facts);
            merging.extractions.push(part.extraction);
            merging.contexts.push(part.contexts);
            merging
        });
        Merging {
            facts: split.facts,
            extraction: split.extractions.into_iter().collect(),
            contexts: split.contexts.into_iter().collect(),
        }
    }
}
