mod operators;

use std::fmt;

use body_lifts::Body;
use lint_layout::Name;
use newtype::Counter;
use transition_facts::{Ground, Mutation, Wrapping};

use crate::algebra::Totality;
use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Node};
use crate::listed::Listed;
use operators::Operators;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Acting<'found> {
    subject: Node<'found>,
    held: Count,
    parameters: Vec<End<'found>>,
}

impl<'found> Acting<'found> {
    pub(crate) fn of_edge(edge: &Edge<'found, '_>) -> Option<Self> {
        let ends = edge.ends();
        let function = edge.transition().function();
        ends.subject(function.mutation(), function.wrapping())
            .and_then(|subject| Acting::unwrapped(subject, ends.takes(), function.takings()))
            .map(|subject| Acting::about(subject, ends.takes()))
            .filter(|acting| acting.grounded().is_some())
            .filter(|acting| acting.admitted(edge.fixed_by()).is_some())
    }

    fn unwrapped(subject: Node<'found>, takes: &[End<'found>], takings: &[Wrapping]) -> Option<Node<'found>> {
        let wrapped = takes
            .iter()
            .zip(takings)
            .filter(|(_, taking)| **taking != Wrapping::Plain)
            .find(|(taken, _)| taken.node() == Some(&subject));
        wrapped.map_or(Some(subject), |_| None)
    }

    fn grounded(&self) -> Option<Count> {
        self.held.nonzero().or_else(|| self.pure().map(|_| self.held))
    }

    fn admitted(&self, fixed_by: Option<&Name>) -> Option<&Self> {
        fixed_by.map_or(Some(self), |fixing| Operators::named(fixing).and(self.pure()))
    }

    fn about(subject: Node<'found>, takes: &[End<'found>]) -> Self {
        let (held, parameters): (Vec<End<'found>>, Vec<End<'found>>) =
            takes.iter().copied().partition(|taken| taken.node() == Some(&subject));
        Acting { subject, held: Count::new(held.len()), parameters }
    }

    #[must_use]
    pub fn pure(&self) -> Option<&Self> {
        Some(self).filter(|acting| acting.parameters.is_empty())
    }

    #[must_use]
    pub const fn subject(&self) -> &Node<'found> {
        &self.subject
    }
}

impl fmt::Display for Acting<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let held = std::iter::repeat_n(End::domain(self.subject), self.held.count());
        let taken = Listed::<_, ','>::joined(held.chain(self.parameters.iter().copied()));
        write!(formatter, "({taken})")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Effect;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Form<'found> {
    Operation(Acting<'found>),
    Partial(Acting<'found>),
    Transition,
    Port,
    Lift,
    Store,
}

impl<'found> Form<'found> {
    pub(crate) fn of_edge(edge: &Edge<'found, '_>) -> Self {
        let transition = edge.transition();
        let function = transition.function();
        let ground = *AsRef::<Ground>::as_ref(transition);
        Form::lifted(function.body())
            .or_else(|| Form::through_store(ground, function.mutation()))
            .unwrap_or_else(|| Form::acted(edge, ground))
    }

    const fn lifted(body: Body) -> Option<Self> {
        match body {
            Body::Mapped | Body::Focused | Body::Routed => Some(Form::Lift),
            Body::Written => None,
        }
    }

    fn through_store(ground: Ground, mutation: Mutation) -> Option<Self> {
        matches!((ground, mutation), (Ground::Store, Mutation::ThroughSelf)).then_some(Form::Store)
    }

    fn acted(edge: &Edge<'found, '_>, ground: Ground) -> Self {
        Totality::of_wrapping(edge.transition().function().wrapping())
            .zip(Acting::of_edge(edge))
            .map_or_else(|| Form::beyond(ground), |(totality, acting)| totality.shaping(acting))
    }

    const fn beyond(ground: Ground) -> Self {
        match ground {
            Ground::Ports => Form::Port,
            Ground::Core | Ground::Boundary | Ground::Store => Form::Transition,
        }
    }

    pub(crate) fn sorted<Sorted>(&self, written: Sorted, lifted: Sorted, stored: Sorted) -> Sorted {
        match self {
            Form::Operation(_) | Form::Partial(_) | Form::Transition | Form::Port => written,
            Form::Lift => lifted,
            Form::Store => stored,
        }
    }

    #[must_use]
    pub fn written(&self) -> Option<&Self> {
        self.sorted(Some(self), None, None)
    }

    pub(crate) const fn ported(&self) -> Option<Effect> {
        match self {
            Form::Port => Some(Effect),
            Form::Operation(_) | Form::Partial(_) | Form::Transition | Form::Lift | Form::Store => None,
        }
    }

    #[must_use]
    pub const fn algebraic(&self) -> Option<&Acting<'found>> {
        match self {
            Form::Operation(acting) | Form::Partial(acting) => Some(acting),
            Form::Transition | Form::Port | Form::Lift | Form::Store => None,
        }
    }
}
