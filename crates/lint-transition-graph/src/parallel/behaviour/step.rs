use crate::edge::Edge;
use crate::end::End;
use crate::form::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Step<'found> {
    role: Role,
    outside: Option<End<'found>>,
}

impl<'found> Step<'found> {
    pub(super) fn read(edge: &Edge<'found, '_>) -> Option<Step<'found>> {
        let gives: &End<'found> = edge.ends().as_ref();
        let role = edge.form().role();
        let outside = gives.node().map_or(Some(*gives), |_| None);
        edge.transition().context().trait_name().map_or(Some(Step { role, outside }), |_| None)
    }
}
