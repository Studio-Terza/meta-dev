mod operators;

use crate::transitions::handle::Handle;
use crate::transitions::{Ground, Wrapping};
use body_lifts::Body;
use graph::Mutation;
use lint_layout::Name;
use newtype::Counter;

use crate::algebra::Totality;
use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Node};
use operators::Operators;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
    Written,
    Operation,
    Port,
    Lift,
    Store,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Acting<'found> {
    subject: Node<'found>,
    held: Count,
    parameters: Vec<End<'found>>,
}

#[derive(Clone, Copy, Debug)]
struct Unwrapped<'found>(Node<'found>);

#[derive(Debug)]
struct Candidate<'found> {
    subject: Node<'found>,
    held: Count,
    parameters: Vec<End<'found>>,
}

impl<'found> Candidate<'found> {
    fn about(unwrapped: Unwrapped<'found>, given: &End<'found>, takes: &[End<'found>]) -> Candidate<'found> {
        let (held, parameters): (Vec<End<'found>>, Vec<End<'found>>) =
            takes.iter().copied().partition(|taken| taken.alike(given).is_some());
        Candidate { subject: unwrapped.0, held: Count::new(held.len()), parameters }
    }

    fn admitted(self, fixed_by: Option<&Name>) -> Option<Acting<'found>> {
        let operating = fixed_by.map(|fixing| Operators::named(fixing).is_some_and(|_| self.parameters.is_empty()));
        let held = Some(self.held);
        let grounded = self.parameters.first().map_or(held, |_| held.filter(|count| *count > Count::zero()));
        grounded.filter(|_| operating.is_none_or(|operator| operator)).map(|count| Acting {
            subject: self.subject,
            held: count,
            parameters: self.parameters,
        })
    }
}

impl<'found> Acting<'found> {
    pub(crate) fn of_edge(edge: &Edge<'found, '_>) -> Option<Acting<'found>> {
        let ends = edge.ends();
        let function = edge.transition().function();
        let subject = ends.subject(function.mutation(), function.wrapping())?;
        subject
            .node()
            .copied()
            .and_then(|node| Acting::unwrapped(node, ends.takes(), function.takings()))
            .map(|unwrapped| Candidate::about(unwrapped, subject, ends.takes()))
            .and_then(|candidate| candidate.admitted(edge.fixed_by()))
    }

    fn unwrapped(subject: Node<'found>, takes: &[End<'found>], takings: &[Wrapping]) -> Option<Unwrapped<'found>> {
        let wrapped = takes
            .iter()
            .zip(takings)
            .filter(|(_, taking)| **taking != Wrapping::Plain)
            .find(|(taken, _)| taken.node() == Some(&subject));
        wrapped.map_or(Some(Unwrapped(subject)), |_| None)
    }

    pub(crate) fn held(&self) -> Count {
        self.held
    }

    #[must_use]
    pub fn parameters(&self) -> &[End<'found>] {
        &self.parameters
    }

    #[must_use]
    pub fn subject(&self) -> &Node<'found> {
        &self.subject
    }
}

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
    pub(crate) fn of_edge(edge: &Edge<'found, '_>) -> Form<'found> {
        let transition = edge.transition();
        let function = transition.function();
        let ground = *AsRef::<Ground>::as_ref(transition);
        Form::lifted(function.body())
            .or_else(|| Form::through_store(ground, function.mutation(), function.held()))
            .unwrap_or_else(|| Form::acted(edge, ground))
    }

    fn lifted(body: Body) -> Option<Form<'found>> {
        match body {
            Body::Mapped | Body::Focused | Body::Routed => Some(Form::Lift),
            Body::Written | Body::Built => None,
        }
    }

    fn through_store(ground: Ground, mutation: Mutation, handle: Handle) -> Option<Form<'found>> {
        let owned = matches!((ground, mutation), (Ground::Store, Mutation::ThroughSelf));
        handle.operating(mutation, ground).or(owned.then_some(())).map(|()| Form::Store)
    }

    fn acted(edge: &Edge<'found, '_>, ground: Ground) -> Form<'found> {
        Totality::of_wrapping(edge.transition().function().wrapping())
            .zip(Acting::of_edge(edge))
            .map_or_else(|| Form::beyond(ground), |(totality, acting)| totality.shaping(acting))
    }

    fn beyond(ground: Ground) -> Form<'found> {
        match ground {
            Ground::Ports => Form::Port,
            Ground::Core | Ground::Boundary | Ground::Store => Form::Transition,
        }
    }

    pub(crate) fn role(&self) -> Role {
        match self {
            Form::Operation(_) | Form::Partial(_) => Role::Operation,
            Form::Port => Role::Port,
            Form::Lift => Role::Lift,
            Form::Store => Role::Store,
            Form::Transition => Role::Written,
        }
    }

    #[must_use]
    pub fn algebraic(&self) -> Option<&Acting<'found>> {
        match self {
            Form::Operation(acting) | Form::Partial(acting) => Some(acting),
            Form::Transition | Form::Port | Form::Lift | Form::Store => None,
        }
    }
}
