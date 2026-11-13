use std::collections::BTreeMap;

use super::{Bindings, Bound};
use lint_sweep::Descend;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BodyId(usize);

#[derive(Clone, Copy, Debug)]
struct Body {
    id: BodyId,
    start: usize,
    end: usize,
}

#[derive(Debug, Default)]
pub struct Scopes {
    constants: Vec<Bound>,
    locals: Vec<Bound>,
    count: usize,
    open: Vec<Body>,
    closed: BTreeMap<BodyId, Body>,
}

impl Scopes {
    pub fn opened(&mut self) -> Descend {
        let start = self.locals.len();
        self.open.push(Body { id: BodyId(self.count), start, end: start });
        self.count += 1;
        Descend::Into
    }

    pub fn shut(&mut self) {
        let end = self.locals.len();
        self.closed.extend(self.open.pop().map(|body| (body.id, Body { end, ..body })));
    }

    pub fn constant(&mut self, bound: Bound) {
        self.constants.push(bound);
    }

    pub fn local(&mut self, bound: Option<Bound>) {
        self.locals.extend(bound);
    }

    pub fn current(&self) -> Option<BodyId> {
        self.open.last().map(|body| body.id)
    }

    pub fn constants(&self) -> Bindings<'_> {
        self.constants.iter().collect()
    }

    pub fn locals(&self, body: Option<BodyId>) -> Bindings<'_> {
        body.and_then(|id| self.closed.get(&id))
            .and_then(|found| self.locals.get(found.start..found.end))
            .map_or_default(|bound| bound.iter().collect())
    }
}
