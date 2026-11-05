use syn::Pat;

use lint_layout::Name;

use super::{Locals, Quantifiers};
use crate::claim::{Binder, Guard, Premise, Term};

#[derive(Clone, Debug, Default)]
pub(crate) struct Frame<'file> {
    locals: Locals<'file>,
    quantifiers: Quantifiers,
    guard: Guard,
}

impl<'file> Frame<'file> {
    pub(crate) fn seeded(quantifiers: &Quantifiers, guard: Guard) -> Frame<'file> {
        Frame { locals: Locals::default(), quantifiers: quantifiers.clone(), guard }
    }

    pub(crate) fn locals(&self) -> &Locals<'file> {
        &self.locals
    }

    pub(crate) fn locals_mut(&mut self) -> &mut Locals<'file> {
        &mut self.locals
    }

    pub(crate) fn quantifiers(&self) -> &Quantifiers {
        &self.quantifiers
    }

    pub(crate) fn guard(&self) -> &Guard {
        &self.guard
    }

    pub(crate) fn guarded(&mut self, guard: Guard) {
        self.guard = guard;
    }

    pub(crate) fn localized(&mut self, locals: Locals<'file>) {
        self.locals = locals;
    }

    pub(crate) fn restated(&mut self, name: Name, term: Term) {
        self.locals = self.locals.restated(name, term);
    }

    pub(crate) fn settled(&mut self, inner: &Frame<'file>) {
        self.locals = self.locals.unsettled(&inner.locals);
    }

    pub(crate) fn quantified(&mut self, binder: Binder, pattern: &Pat) {
        self.quantifiers = self.quantifiers.quantified(binder, pattern);
    }

    pub(crate) fn assumed(&mut self, premise: &Premise) {
        let narrowed =
            premise.bounding().and_then(|bounding| self.quantifiers.narrowed(bounding.depth(), bounding.ranges()));
        *self = narrowed.map_or_else(
            || Frame { guard: self.guard.conjoined(premise.clone()), ..self.clone() },
            |quantifiers| Frame { quantifiers, ..self.clone() },
        );
    }
}
