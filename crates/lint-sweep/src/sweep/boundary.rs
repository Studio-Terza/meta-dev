use lint_core::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Index(usize);

impl Index {
    pub(super) fn listening(kinds: &[&[Kind]], kind: Kind) -> Vec<Index> {
        let heard = kinds.iter().enumerate().filter(|(_, wanted)| wanted.contains(&kind));
        heard.map(|(index, _)| Index(index)).collect()
    }

    pub(super) fn entry_mut<Value>(self, values: &mut [Value]) -> Option<&mut Value> {
        values.get_mut(self.0)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Muting {
    muted: Vec<Option<usize>>,
    awake: usize,
    depth: usize,
}

impl Muting {
    pub(super) fn new(watchers: usize) -> Muting {
        Muting { muted: vec![None; watchers], awake: watchers, depth: 0 }
    }

    pub(super) fn awake(&self) -> Option<usize> {
        Some(self.awake).filter(|awake| *awake > 0)
    }

    pub(super) fn entering(&mut self) {
        self.depth += 1;
    }

    pub(super) fn listening(&self, index: Index) -> Option<Index> {
        self.muted.get(index.0).copied().flatten().map_or(Some(index), |_| None)
    }

    pub(super) fn silence(&mut self, index: Index) {
        let depth = self.depth;
        let silenced = self.muted.get_mut(index.0).map(|muted| muted.replace(depth));
        self.awake -= silenced.map_or(0, |_| 1);
    }

    pub(super) fn leaving(&mut self, called: &[Index]) {
        let depth = self.depth;
        let woken = called.iter().filter_map(|index| self.muted.get_mut(index.0)?.take_if(|muted| *muted == depth));
        self.awake += woken.count();
        self.depth -= 1;
    }

    pub(super) fn hushed(&mut self, kept: &[Index]) -> Muting {
        let depth = self.depth;
        let hushed = Muting {
            muted: (0..self.muted.len())
                .map(|index| kept.iter().find(|held| held.0 == index).map_or(Some(depth), |_| None))
                .collect(),
            awake: kept.len(),
            depth,
        };
        std::mem::replace(self, hushed)
    }

    pub(super) fn restored(&mut self, saved: Muting) {
        *self = saved;
    }
}
