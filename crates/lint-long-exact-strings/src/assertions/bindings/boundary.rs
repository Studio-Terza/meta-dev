use lint_core::Extended;
use std::collections::BTreeMap;

use super::{Bindings, Bound};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BodyId(usize);

impl BodyId {
    pub fn locals(self, scopes: &Scopes) -> Bindings<'_> {
        scopes
            .closed
            .get(&self)
            .and_then(|found| scopes.locals.get(found.start..found.end))
            .map_or_default(|bound| bound.iter().collect())
    }
}

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

#[derive(Debug)]
pub enum Step {
    Open,
    Shut,
    Constant(Bound),
    Local(Option<Bound>),
}

impl Scopes {
    #[must_use]
    pub fn stepped(self, step: Step) -> Scopes {
        match step {
            Step::Open => Scopes {
                open: self
                    .open
                    .iter()
                    .copied()
                    .chain([Body { id: BodyId(self.count), start: self.locals.len(), end: self.locals.len() }])
                    .collect(),
                count: self.count + 1,
                ..self
            },
            Step::Shut => Scopes {
                closed: self
                    .closed
                    .into_iter()
                    .chain(self.open.last().map(|body| (body.id, Body { end: self.locals.len(), ..*body })))
                    .collect(),
                open: self.open.iter().take(self.open.len().saturating_sub(1)).copied().collect(),
                ..self
            },
            Step::Constant(bound) => Scopes { constants: self.constants.extended([bound]), ..self },
            Step::Local(bound) => Scopes { locals: self.locals.extended(bound), ..self },
        }
    }

    pub fn current(&self) -> Option<BodyId> {
        self.open.last().map(|body| body.id)
    }

    pub fn constants(&self) -> Bindings<'_> {
        self.constants.iter().collect()
    }
}
