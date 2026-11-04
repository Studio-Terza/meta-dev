use std::cmp::Ordering;

use equivalence::{Agreeing, Equivalence, Erased, Laws};
use law_facts::Property;
use lint_layout::Name;

use crate::edge::Edge;
use crate::end::End;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role<'found> {
    Written,
    Operation(Option<&'found Name>),
    Port,
    Lift,
    Store,
}

impl<'found> Role<'found> {
    pub(super) fn read(edge: &Edge<'found, '_>, equivalence: Equivalence) -> Option<Role<'found>> {
        let form = edge.form();
        let written = Self::written(edge.transition().context().trait_name(), equivalence);
        let effect = form.ported().map(|_| Role::Port);
        form.algebraic().map_or_else(
            || effect.map_or_else(|| form.sorted(written, Some(Role::Lift), Some(Role::Store)), Some),
            |_| Some(Role::Operation(Self::named(edge, equivalence))),
        )
    }

    fn named(edge: &Edge<'found, '_>, equivalence: Equivalence) -> Option<&'found Name> {
        equivalence.erases(Erased::Names).map_or(Some(edge.transition().function().as_ref()), |_| None)
    }

    fn written(trait_name: Option<&Name>, equivalence: Equivalence) -> Option<Role<'found>> {
        trait_name.map_or(Some(Role::Written), |_| equivalence.erases(Erased::TraitLabels).map(|_| Role::Written))
    }
}

#[derive(Clone, Copy, Debug)]
struct Declared<'laws> {
    laws: &'laws [Property],
    mode: Laws,
}

impl PartialEq for Declared<'_> {
    fn eq(&self, _other: &Declared<'_>) -> bool {
        true
    }
}

impl Eq for Declared<'_> {}

impl PartialOrd for Declared<'_> {
    fn partial_cmp(&self, other: &Declared<'_>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Declared<'_> {
    fn cmp(&self, _other: &Declared<'_>) -> Ordering {
        Ordering::Equal
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Step<'found, 'laws> {
    role: Role<'found>,
    outside: Option<End<'found>>,
    laws: &'laws [Property],
    declared: Declared<'laws>,
}

impl<'found, 'laws> Agreeing for Step<'found, 'laws> {
    fn agrees<'other>(&self, other: &'other Step<'found, 'laws>) -> Option<&'other Step<'found, 'laws>> {
        self.declared.mode.agree(self.declared.laws, other.declared.laws).map(|_| other)
    }
}

impl<'found, 'laws> Step<'found, 'laws> {
    pub(super) fn read(
        edge: &Edge<'found, '_>,
        laws: &'laws [Property],
        equivalence: Equivalence,
    ) -> Option<Step<'found, 'laws>> {
        let gives: &End<'found> = edge.ends().as_ref();
        Role::read(edge, equivalence).map(|role| Step {
            role,
            outside: gives.node().map_or(Some(*gives), |_| None),
            laws: equivalence.laws().kept(laws),
            declared: Declared { laws, mode: equivalence.laws() },
        })
    }
}
