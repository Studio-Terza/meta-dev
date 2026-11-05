use std::collections::BTreeMap;

use syn::Pat;

use lint_layout::Name;

use crate::boundary::Position;
use crate::claim::{Binder, Binders, Callee, Depth, Term};

#[derive(Clone, Debug, Default)]
pub(crate) struct Quantifiers {
    binders: Binders,
    names: BTreeMap<Name, Term>,
}

impl Quantifiers {
    pub(crate) fn binders(&self) -> &Binders {
        &self.binders
    }

    pub(crate) fn named(&self, name: &Name) -> Option<&Term> {
        self.names.get(name)
    }

    pub(crate) fn quantified(&self, binder: Binder, pattern: &Pat) -> Quantifiers {
        let depth = Depth::new(self.binders.len());
        let mut next = self.clone();
        next.binders.insert(depth, binder);
        next.bind(pattern, Term::Bound(depth));
        next
    }

    pub(crate) fn drawn(&self, binder: Binder) -> Quantifiers {
        let depth = Depth::new(self.binders.len());
        let mut next = self.clone();
        next.binders.insert(depth, binder);
        next
    }

    pub(crate) fn bound(&self) -> impl Iterator<Item = Term> + '_ {
        self.binders.keys().map(|depth| Term::Bound(*depth))
    }

    fn bind(&mut self, pattern: &Pat, value: Term) {
        match pattern {
            Pat::Ident(named) => {
                self.names.insert(Name::from(&named.ident), value);
            }
            Pat::Tuple(tuple) => tuple.elems.iter().enumerate().for_each(|(index, inner)| {
                let field = Callee::Field(Name::from(Position::counted(index)));
                self.bind(inner, field.folded(vec![value.clone()]));
            }),
            Pat::Reference(reference) => self.bind(&reference.pat, value),
            Pat::Paren(inner) => self.bind(&inner.pat, value),
            Pat::Type(typed) => self.bind(&typed.pat, value),
            _ => (),
        }
    }
}
