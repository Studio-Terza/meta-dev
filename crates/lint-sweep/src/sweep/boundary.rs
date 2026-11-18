use std::collections::{BTreeMap, BTreeSet};

use lint_core::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Index(usize);

impl Index {
    pub(super) fn listening(kinds: &[&[Kind]]) -> BTreeMap<Kind, Vec<Index>> {
        let heard =
            kinds.iter().enumerate().flat_map(|(index, wanted)| wanted.iter().map(move |kind| (*kind, Index(index))));
        let mut listening = heard.fold(BTreeMap::new(), |mut by: BTreeMap<Kind, Vec<Index>>, (kind, index)| {
            by.entry(kind).or_default().push(index);
            by
        });
        listening.values_mut().for_each(Vec::dedup);
        listening
    }

    pub(super) fn entry_mut<Value>(self, values: &mut [Value]) -> Option<&mut Value> {
        values.get_mut(self.0)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Muting {
    muted: Vec<Option<usize>>,
    awake: BTreeSet<usize>,
    depth: usize,
}

impl Muting {
    pub(super) fn new(watchers: usize) -> Muting {
        Muting { muted: vec![None; watchers], awake: (0..watchers).collect(), depth: 0 }
    }

    pub(super) fn awake(&self) -> Option<usize> {
        self.awake.first().copied()
    }

    pub(super) fn entering(&mut self) {
        self.depth += 1;
    }

    pub(super) fn silence(&mut self, index: Index) {
        let depth = self.depth;
        let silenced = self.muted.get_mut(index.0).map(|muted| muted.replace(depth));
        silenced.map(|_| self.awake.remove(&index.0));
    }

    pub(super) fn heard(&self, index: Index) -> Option<Index> {
        let muted = self.muted.get(index.0).copied().flatten();
        muted.is_none_or(|depth| depth == self.depth).then_some(index)
    }

    pub(super) fn leaving(&mut self, listening: &[Index]) {
        let depth = self.depth;
        let woken: Vec<usize> = listening
            .iter()
            .filter_map(|index| self.muted.get_mut(index.0)?.take_if(|muted| *muted == depth).map(|_| index.0))
            .collect();
        self.awake.extend(woken);
        self.depth -= 1;
    }

    pub(super) fn hushed(&mut self, kept: &[Index]) -> Muting {
        let depth = self.depth;
        let hushed = Muting {
            muted: (0..self.muted.len())
                .map(|index| kept.iter().find(|held| held.0 == index).map_or(Some(depth), |_| None))
                .collect(),
            awake: kept.iter().map(|held| held.0).collect(),
            depth,
        };
        std::mem::replace(self, hushed)
    }

    pub(super) fn restored(&mut self, saved: Muting) {
        *self = saved;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member(usize);

impl Member {
    pub const FIRST: Member = Member(0);

    #[must_use]
    pub const fn new(index: usize) -> Member {
        Member(index)
    }

    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}
