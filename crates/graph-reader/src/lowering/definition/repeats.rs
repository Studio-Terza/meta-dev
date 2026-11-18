mod boundary;

use std::collections::{BTreeMap, BTreeSet};

use syntax::Spanned;
use syntax::{Block, Expr, ExprClosure, ExprField, ExprMethodCall, ExprPath, Signature};

use algebra::{Monoid, Semigroup};
use graph::{Looping, ParamIndex, Reached, Repeating, Repetition};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};

use super::repetitions::{Bound, PerElement, SelfField};
use boundary::Extent;

#[derive(Debug)]
struct Context {
    extent: Extent,
    repeating: Repeating,
}

#[derive(Debug)]
struct Shadow {
    extent: Extent,
    names: BTreeSet<Name>,
}

#[derive(Clone, Debug)]
enum Touched {
    Parameter(Name),
    Field(Reached),
}

#[derive(Debug)]
struct Candidate {
    extent: Extent,
    touched: Touched,
    line: LineNumber,
    method: Option<Name>,
}

#[derive(Debug, Default)]
pub(crate) struct Repeats {
    contexts: Vec<Context>,
    shadows: Vec<Shadow>,
    candidates: Vec<Candidate>,
    receivers: Vec<Extent>,
}

#[derive(Debug)]
pub(crate) struct Parameters(BTreeMap<Name, ParamIndex>);

impl Semigroup for Repeats {
    fn combine(self, other: Repeats) -> Repeats {
        Repeats {
            contexts: self.contexts.combine(other.contexts),
            shadows: self.shadows.combine(other.shadows),
            candidates: self.candidates.combine(other.candidates),
            receivers: self.receivers.combine(other.receivers),
        }
    }
}

impl Monoid for Repeats {}

impl Repeats {
    pub(crate) fn looped(body: &Block, looping: Looping) -> Repeats {
        let context = Context { extent: Extent::from(body.span()), repeating: Repeating::Loop(looping) };
        Repeats { contexts: vec![context], ..Repeats::default() }
    }

    pub(crate) fn closure(closure: &ExprClosure) -> Repeats {
        let names: BTreeSet<Name> = closure.inputs.iter().filter_map(Bound::pattern).collect();
        Repeats { shadows: vec![Shadow { extent: Extent::from(closure.span()), names }], ..Repeats::default() }
    }

    pub(crate) fn path(path: &ExprPath) -> Repeats {
        let touched = Touched::of_path(path);
        let line = LineNumber::from(path.path.span());
        let candidates =
            touched.map(|found| Candidate { extent: Extent::from(path.span()), touched: found, line, method: None });
        Repeats { candidates: candidates.into_iter().collect(), ..Repeats::default() }
    }

    pub(crate) fn field(field: &ExprField) -> Repeats {
        let line = LineNumber::from(field.member.span());
        let candidates = SelfField::reached(field).map(|reached| Candidate {
            extent: Extent::from(field.span()),
            touched: Touched::Field(reached),
            line,
            method: None,
        });
        Repeats { candidates: candidates.into_iter().collect(), ..Repeats::default() }
    }

    pub(crate) fn method(call: &ExprMethodCall) -> Repeats {
        let adaptor = PerElement::adaptor(call).map(Repeating::Closure);
        let contexts = call
            .args
            .iter()
            .filter(|argument| Syntax::closure(argument).is_some())
            .filter_map(|argument| Some(Context { extent: Extent::from(argument.span()), repeating: adaptor.clone()? }))
            .collect();
        let receiver = Repeats::receiving(&call.receiver, Name::from(&call.method));
        Repeats { contexts, ..receiver }
    }

    fn receiving(receiver: &Expr, method: Name) -> Repeats {
        let touched = Touched::of_expression(receiver);
        let extent = Extent::from(receiver.span());
        let candidates = touched.map(|found| Candidate {
            extent,
            touched: found,
            line: LineNumber::from(receiver.span()),
            method: Some(method),
        });
        Repeats { candidates: candidates.into_iter().collect(), ..Repeats::default() }
    }

    pub(crate) fn found(self, parameters: &Parameters) -> Vec<Repetition> {
        let mut ordered: Vec<&Candidate> = self.candidates.iter().collect();
        ordered.sort_by_key(|candidate| candidate.extent.preorder());
        let settled: Vec<Settled<'_>> = ordered
            .into_iter()
            .map(|candidate| Settled { candidate, reached: self.reached(candidate, parameters) })
            .collect();
        let skipped: Vec<Extent> = settled.iter().filter_map(Settled::skipping).collect();
        settled
            .into_iter()
            .filter(|found| found.kept(&skipped).is_some())
            .filter_map(|found| self.repetition(found))
            .collect()
    }

    fn reached(&self, candidate: &Candidate, parameters: &Parameters) -> Option<Reached> {
        match &candidate.touched {
            Touched::Parameter(name) => parameters.reached(name, &self.shadows, candidate.extent),
            Touched::Field(reached) => Some(reached.clone()),
        }
    }

    fn repetition(&self, found: Settled<'_>) -> Option<Repetition> {
        let candidate = found.candidate;
        let holding = self.contexts.iter().filter(|context| context.extent.holds(candidate.extent).is_some());
        let outermost = holding.min_by_key(|context| context.extent.preorder())?;
        Some(Repetition::new(found.reached?, candidate.line, outermost.repeating.clone(), candidate.method.clone()))
    }
}

#[derive(Debug)]
struct Settled<'repeats> {
    candidate: &'repeats Candidate,
    reached: Option<Reached>,
}

impl Settled<'_> {
    fn skipping(&self) -> Option<Extent> {
        self.candidate.method.as_ref()?;
        self.reached.as_ref().map(|_| self.candidate.extent)
    }

    fn kept(&self, skipped: &[Extent]) -> Option<()> {
        let received = self.candidate.method.as_ref().map(|_| ());
        let inside = skipped.iter().find(|extent| extent.holds(self.candidate.extent).is_some());
        received.or_else(|| inside.map_or(Some(()), |_| None))
    }
}

impl Parameters {
    pub(crate) fn of_signature(signature: &Signature) -> Parameters {
        Parameters(
            signature
                .inputs
                .iter()
                .enumerate()
                .filter_map(|(index, input)| Some((Bound::named(input)?, ParamIndex::from(index))))
                .collect(),
        )
    }

    fn reached(&self, name: &Name, shadows: &[Shadow], extent: Extent) -> Option<Reached> {
        let shadowing = shadows.iter().filter(|shadow| shadow.extent.holds(extent).is_some());
        let shadowed = shadowing.into_iter().find(|shadow| shadow.names.contains(name));
        Some(()).filter(|()| shadowed.is_none())?;
        let index = *self.0.get(name)?;
        Some(Reached::Parameter { index, name: name.clone() })
    }
}

impl Touched {
    fn of_expression(expression: &Expr) -> Option<Touched> {
        match expression {
            Expr::Path(path) => Touched::of_path(path),
            Expr::Field(field) => SelfField::reached(field).map(Touched::Field),
            _ => None,
        }
    }

    fn of_path(path: &ExprPath) -> Option<Touched> {
        path.path.get_ident().map(|ident| Touched::Parameter(Name::from(ident)))
    }
}
