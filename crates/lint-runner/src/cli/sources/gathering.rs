use std::mem;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use lint_core::{FileHead, Parsed, Readout, SourceFile, Unwalked};
use lint_sweep::Gathers;

use super::super::timings::{Measured, Timing};
use super::contexts::{Contexts, Contributed};
use crate::cli::ignored::Ignored;
use invariants::{Ignoring, Unpoisoned};

use super::super::boundary::FileOrder;

#[derive(Debug)]
struct Claim<'read, 'run> {
    order: FileOrder,
    file: &'read SourceFile,
    head: &'run FileHead,
    unwalked: Unwalked,
}

impl<'read, 'run> Claim<'read, 'run> {
    fn gathered<Part, Filed: Gathers<Part>>(self, starting: &impl Fn(Claimed<'read>) -> Filed) -> Joined<'run, Part> {
        let parsed = Parsed::new(self.file, self.head);
        let claimed = Claimed { order: self.order, file: self.file };
        let filing = starting(claimed);
        let contexts = Contributed::joined(parsed, self.unwalked);
        Joined { gathered: Gathered { head: self.head, contexts }, filed: filing.gathered(self.file) }
    }
}

#[derive(Debug)]
struct Joined<'run, Part> {
    gathered: Gathered<'run>,
    filed: Part,
}

#[derive(Debug)]
struct Gathered<'run> {
    head: &'run FileHead,
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

#[derive(Debug)]
pub(crate) struct Merging<'run>(Vec<Gathered<'run>>);

impl<'run> Merging<'run> {
    pub(crate) fn settled(
        self,
        gathering: &Gathering<'run>,
        contexts: impl FnOnce(Vec<Contributed<'run>>) -> Measured<Contexts<'run>>,
    ) -> Timing {
        let mut sorted = self.0;
        sorted.sort_by(|earlier, later| AsRef::<Path>::as_ref(earlier.head).cmp(AsRef::<Path>::as_ref(later.head)));
        contexts(sorted.into_iter().map(|part| part.contexts).collect()).kept(&gathering.contexts)
    }
}

#[derive(Debug)]
pub(crate) struct Gathering<'run> {
    heads: &'run OnceLock<Vec<FileHead>>,
    deposited: Mutex<Vec<Gathered<'run>>>,
    contexts: OnceLock<Contexts<'run>>,
}

impl<'run> Gathering<'run> {
    pub(crate) fn new(heads: &'run OnceLock<Vec<FileHead>>) -> Gathering<'run> {
        Gathering { heads, deposited: Mutex::default(), contexts: OnceLock::new() }
    }

    pub(crate) fn gathered<'read, Part, Filed: Gathers<Part>>(
        &self,
        readout: &'read Readout,
        table: &[FileHead],
        starting: impl Fn(Claimed<'read>) -> Filed,
    ) -> Share<'run, Part> {
        let heads = self.heads.get_or_init(|| table.to_vec());
        let claims: Vec<Claim<'read, 'run>> = readout
            .files()
            .iter()
            .zip(readout.unwalked())
            .filter_map(|(file, unwalked)| {
                let index = heads
                    .binary_search_by(|head| AsRef::<Path>::as_ref(head).cmp(AsRef::<Path>::as_ref(file.head())))
                    .ignoring(Ignored::Unclaimed)?;
                Some(Claim { order: FileOrder::from(index), file, head: heads.get(index)?, unwalked })
            })
            .collect();
        let joined: Vec<Joined<'run, Part>> = claims.into_iter().map(|claim| claim.gathered(&starting)).collect();
        let (gathered, filed): (Vec<Gathered<'run>>, Vec<Part>) =
            joined.into_iter().map(|found| (found.gathered, found.filed)).unzip();
        Share { merging: self.deposited(gathered, heads), filed }
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
