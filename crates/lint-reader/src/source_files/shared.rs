mod assembly;
mod boundary;
mod declared;
mod exploring;

use std::collections::BTreeMap;
use std::mem;
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

use file_ports::Files;
use lint_core::{FileHead, Readout, SourceFile};
use lint_layout::Failure;

use super::Stage;
use super::located::Located;
use super::modules::Modules;
use assembly::Assembly;
use boundary::Expected;
pub use boundary::Partition;
use declared::Declared;
use exploring::{Exploring, Queued};

#[derive(Debug)]
pub struct SharedRead<'tree> {
    stage: &'tree Stage<'tree>,
    exploring: Mutex<Exploring<'tree>>,
    changed: Condvar,
    assembly: OnceLock<Assembly<'tree>>,
    heads: Mutex<Vec<FileHead>>,
    table: OnceLock<Vec<FileHead>>,
}

impl<'tree> SharedRead<'tree> {
    #[must_use]
    pub fn new(stage: &'tree Stage<'tree>) -> SharedRead<'tree> {
        let starts = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().flat_map(|entry| entry.starts(tree)));
        SharedRead {
            stage,
            exploring: Mutex::new(Exploring::started(starts.collect())),
            changed: Condvar::new(),
            assembly: OnceLock::new(),
            heads: Mutex::default(),
            table: OnceLock::new(),
        }
    }

    pub fn read(&self, files: &impl Files, share: Partition) -> Readout {
        let explorer = Explorer { shared: self, files, share, parsed: BTreeMap::new() };
        let claims = std::iter::from_fn(|| self.claimed(share));
        let parsed = claims.fold(explorer, |exploring, queued| exploring.explored(&queued)).parsed;
        let assembly = self.assembly.get_or_init(|| self.locked().assembled(self.stage));
        let readout = assembly.readout(parsed, share);
        self.deposited(readout.files().iter().map(|file| file.head().clone()).collect(), assembly.expected());
        readout
    }

    #[must_use]
    pub fn heads(&self) -> &[FileHead] {
        self.table.wait()
    }

    fn claimed(&self, share: Partition) -> Option<Queued<'tree>> {
        let guard = self.locked();
        let mut ready = self
            .changed
            .wait_while(guard, |state| state.waiting(share).is_some())
            .unwrap_or_else(PoisonError::into_inner);
        let claimed = ready.claimed(share);
        self.changed.notify_all();
        claimed
    }

    fn locked(&self) -> MutexGuard<'_, Exploring<'tree>> {
        self.exploring.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn deposited(&self, heads: Vec<FileHead>, expected: Expected) {
        let mut held = self.heads.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(heads);
        let complete = expected.met(&held).map(|_| mem::take(&mut *held));
        let sorted = complete.map(|mut table| {
            table.sort_by(|earlier, later| earlier.path().cmp(later.path()));
            table
        });
        sorted.map(|table| self.table.get_or_init(|| table));
    }
}

struct Explorer<'shared, 'tree, Port> {
    shared: &'shared SharedRead<'tree>,
    files: &'shared Port,
    share: Partition,
    parsed: BTreeMap<Located<'tree>, SourceFile>,
}

impl<'tree, Port: Files> Explorer<'_, 'tree, Port> {
    fn explored(self, queued: &Queued<'tree>) -> Self {
        let located = queued.node().located();
        let opened = Located::open_many(self.files, &self.shared.stage.packages, std::slice::from_ref(located));
        let parsed = opened.into_iter().map(|found| found.and_then(SourceFile::try_from));
        let settled = parsed.fold(self, |explorer, outcome| explorer.settled(queued, outcome));
        settled.shared.changed.notify_all();
        settled
    }

    fn settled(self, queued: &Queued<'tree>, parsed: Result<SourceFile, Failure>) -> Self {
        match parsed {
            Ok(file) => self.kept(queued, file),
            Err(failure) => self.failed(queued, failure),
        }
    }

    fn kept(mut self, queued: &Queued<'tree>, file: SourceFile) -> Self {
        let node = queued.node();
        let candidates = Modules::candidates(node.located().file(), node.children(), &file.syntax().items);
        let mut state = self.shared.locked();
        let declared = Declared::read(state.learned(self.files, &candidates), node, &file.syntax().items);
        state.settled(queued, Ok(declared), self.share);
        drop(state);
        self.parsed.insert(node.located().clone(), file);
        self
    }

    fn failed(self, queued: &Queued<'tree>, failure: Failure) -> Self {
        self.shared.locked().settled(queued, Err(failure), self.share);
        self
    }
}
