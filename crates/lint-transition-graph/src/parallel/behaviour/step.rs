use crate::edge::Edge;
use crate::end::End;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role {
    Written,
    Operation,
    Port,
    Lift,
    Store,
}

impl Role {
    pub(super) fn read(edge: &Edge<'_, '_>) -> Option<Role> {
        let form = edge.form();
        let effect = form.ported().map(|_| Role::Port);
        let role = form
            .algebraic()
            .map(|_| Role::Operation)
            .or(effect)
            .or_else(|| form.lifting(Role::Lift).map(|lifted| *lifted.as_ref()))
            .or_else(|| form.storing(Role::Store).map(|stored| *stored.as_ref()))
            .unwrap_or(Role::Written);
        edge.transition().context().trait_name().map_or(Some(role), |_| None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Step<'found> {
    role: Role,
    outside: Option<End<'found>>,
}

impl<'found> Step<'found> {
    pub(super) fn read(edge: &Edge<'found, '_>) -> Option<Step<'found>> {
        let gives: &End<'found> = edge.ends().as_ref();
        Role::read(edge).map(|role| Step { role, outside: gives.node().map_or(Some(*gives), |_| None) })
    }
}
