use std::mem;
use std::sync::{Mutex, OnceLock, PoisonError};

use code_model::Model;
use code_reader::{FileFacts, ListedGroup, Listing};
use lint_core::{FileHead, Parsed, Readout, SourceFile};
use lint_transition_graph::Extraction;

use super::super::boundary::Queue;

#[derive(Debug)]
struct Claim<'read, 'run> {
    group: &'read ListedGroup<'read>,
    file: &'read SourceFile,
    head: &'run FileHead,
}

impl<'run> Claim<'_, 'run> {
    fn gathered(&self) -> Gathered<'run> {
        let extraction = Extraction::from(Parsed::new(self.file, self.head));
        Gathered { head: self.head, facts: self.group.lowered(), extraction }
    }
}

#[derive(Debug)]
struct Gathered<'run> {
    head: &'run FileHead,
    facts: Vec<FileFacts>,
    extraction: Extraction<'run>,
}

#[derive(Debug)]
pub(crate) struct Merging<'run> {
    facts: Vec<FileFacts>,
    extraction: Extraction<'run>,
}

#[derive(Debug)]
pub(crate) struct Gathering<'run> {
    heads: &'run OnceLock<Vec<FileHead>>,
    claims: Queue,
    deposited: Mutex<Vec<Gathered<'run>>>,
    model: OnceLock<Option<Model>>,
    extraction: OnceLock<Extraction<'run>>,
}

impl<'run> Gathering<'run> {
    pub(crate) fn new(heads: &'run OnceLock<Vec<FileHead>>) -> Gathering<'run> {
        Gathering {
            heads,
            claims: Queue::default(),
            deposited: Mutex::default(),
            model: OnceLock::new(),
            extraction: OnceLock::new(),
        }
    }

    pub(crate) fn gathered(&self, readout: &Readout) -> Option<Merging<'run>> {
        let heads = self.heads.get_or_init(|| readout.files().iter().map(|file| file.head().clone()).collect());
        let listing = Listing::of_files(readout.files());
        let claims: Vec<Claim<'_, 'run>> = listing
            .groups()
            .iter()
            .zip(readout.files())
            .zip(heads)
            .map(|((group, file), head)| Claim { group, file, head })
            .collect();
        let parts: Vec<Gathered<'run>> = self.claims.claimed(&claims).map(Claim::gathered).collect();
        self.deposited(parts, heads)
    }

    pub(crate) fn settled(&self, merging: Merging<'run>, model: impl FnOnce(Vec<FileFacts>) -> Option<Model>) {
        self.extraction.get_or_init(|| merging.extraction);
        self.model.get_or_init(|| model(merging.facts));
    }

    pub(crate) fn model(&self) -> Option<&Model> {
        self.model.wait().as_ref()
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
        let (facts, extractions): (Vec<Vec<FileFacts>>, Vec<Extraction<'run>>) =
            sorted.into_iter().map(|part| (part.facts, part.extraction)).unzip();
        Merging { facts: facts.into_iter().flatten().collect(), extraction: extractions.into_iter().collect() }
    }
}
