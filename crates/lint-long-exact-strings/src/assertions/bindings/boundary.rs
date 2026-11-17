use syntax::{ItemConst, Local, Macro};

use super::{Bindings, Bound};
use algebra::{Monoid, Semigroup};
use lint_core::Aliases;

#[derive(Debug)]
struct Body {
    start: usize,
    end: usize,
    macros: Vec<Macro>,
}

impl Body {
    fn shifted(self, by: usize) -> Body {
        Body { start: self.start + by, end: self.end + by, macros: self.macros }
    }
}

#[derive(Debug, Default)]
pub struct Judging {
    constants: Vec<Bound>,
    locals: Vec<Bound>,
    loose: Vec<Macro>,
    bodies: Vec<Body>,
}

#[derive(Debug, Default)]
pub struct Asserted {
    aliases: Aliases,
    judging: Judging,
}

impl Semigroup for Judging {
    fn combine(self, other: Judging) -> Judging {
        let shift = self.locals.len();
        let bodies = self.bodies.into_iter().chain(other.bodies.into_iter().map(|body| body.shifted(shift))).collect();
        Judging {
            constants: self.constants.combine(other.constants),
            locals: self.locals.combine(other.locals),
            loose: self.loose.combine(other.loose),
            bodies,
        }
    }
}

impl Monoid for Judging {}

impl Semigroup for Asserted {
    fn combine(self, other: Asserted) -> Asserted {
        Asserted { aliases: self.aliases.combine(other.aliases), judging: self.judging.combine(other.judging) }
    }
}

impl Monoid for Asserted {}

impl Asserted {
    pub fn constant(item: &ItemConst) -> Asserted {
        Asserted {
            judging: Judging { constants: vec![Bound::constant(item)], ..Judging::default() },
            ..Asserted::default()
        }
    }

    pub fn local(local: &Local) -> Asserted {
        Asserted {
            judging: Judging { locals: Bound::local(local).into_iter().collect(), ..Judging::default() },
            ..Asserted::default()
        }
    }

    pub fn met(invocation: &Macro) -> Asserted {
        Asserted { judging: Judging { loose: vec![invocation.clone()], ..Judging::default() }, ..Asserted::default() }
    }

    pub fn aliased(aliases: Aliases) -> Asserted {
        Asserted { aliases, ..Asserted::default() }
    }

    pub fn bodied(self) -> Asserted {
        let Judging { constants, locals, loose, bodies } = self.judging;
        let own = Body { start: 0, end: locals.len(), macros: loose };
        let judging =
            Judging { constants, locals, loose: Vec::new(), bodies: bodies.into_iter().chain([own]).collect() };
        Asserted { aliases: self.aliases, judging }
    }

    pub fn concluded<Out>(self, into: impl FnOnce(Aliases, Judging) -> Out) -> Out {
        into(self.aliases, self.judging)
    }
}

impl Judging {
    pub fn constants(&self) -> Bindings<'_> {
        self.constants.iter().collect()
    }

    pub fn met(&self) -> impl Iterator<Item = Met<'_>> {
        let loose = self.loose.iter().map(|invocation| Met { invocation, locals: Bindings::default() });
        let bodied = self.bodies.iter().flat_map(|body| {
            let locals = self.locals.get(body.start..body.end).unwrap_or_default();
            body.macros.iter().map(move |invocation| Met { invocation, locals: locals.iter().collect() })
        });
        loose.chain(bodied)
    }
}

#[derive(Debug)]
pub struct Met<'judging> {
    invocation: &'judging Macro,
    locals: Bindings<'judging>,
}

impl<'judging> Met<'judging> {
    pub fn judged<Out>(self, into: impl FnOnce(&'judging Macro, Bindings<'judging>) -> Out) -> Out {
        into(self.invocation, self.locals)
    }
}
