use std::fmt;
use std::iter;

use design_model::{
    Confidence, Explicitness, Facet, Node, NodeId, Ordinal, Prefix, Segment, SegmentRelation, Sense, SenseKind,
    SpanRelation, Specificity, Title,
};
use dispatch_ports::Prompt;
use grammar::directive::Quote;
use newtype::Newtype;
use strum::VariantArray;

use crate::verb::{By, Form, New, Refines, Verb};
use crate::{Attempt, Bind, Directive, Job, Label, NodeRef, Said, SegmentRef, Visit};

#[derive(Clone, Copy, Debug)]
struct Briefing<'job> {
    job: &'job Job,
    attempt: &'job Attempt,
}

#[derive(Clone, Copy, Debug)]
struct Choices<Choice: 'static>(&'static [Choice]);

#[derive(Clone, Copy, Debug)]
struct Context<'node>(&'node Node);

#[derive(Clone, Copy, Debug)]
struct Listed<'said>(&'said Said);

#[derive(Clone, Copy, Debug)]
struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
struct Quoted<'title>(&'title Title);

#[derive(Clone, Copy, Debug)]
struct Fresh<'label>(Option<&'label Label>);

struct Rendering<'out, 'formatter>(&'out mut fmt::Formatter<'formatter>);

impl From<&Segment> for SegmentRef {
    fn from(segment: &Segment) -> SegmentRef {
        SegmentRef::new(Prefix::from(segment), Ordinal::from(segment))
    }
}

impl fmt::Display for SegmentRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix: &Prefix = self.as_ref();
        write!(formatter, "{prefix}.{}", Ordinal::from(self))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{}", self.0.into_inner())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_ref())
    }
}

impl fmt::Display for NodeRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bound(Rendering(formatter))
    }
}

impl Bind for Rendering<'_, '_> {
    type Output = fmt::Result;

    fn stored(self, id: NodeId) -> fmt::Result {
        write!(self.0, "{}", Id(id))
    }

    fn labelled(self, label: &Label) -> fmt::Result {
        write!(self.0, "{label}")
    }
}

impl fmt::Display for Fresh<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        New::New.fmt(formatter)?;
        self.0.iter().try_for_each(|label| write!(formatter, " {label}"))
    }
}

impl fmt::Display for Quoted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&Quote::quoting(&self.0.to_string()))
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visited(Rendering(formatter))
    }
}

impl Visit for Rendering<'_, '_> {
    type Output = fmt::Result;

    fn place(self, segment: &SegmentRef, node: &NodeRef, relation: SpanRelation) -> fmt::Result {
        write!(self.0, "{}({segment}, {node}, {relation})", Verb::Place)
    }

    fn sense(self, segment: &SegmentRef, sense: Sense) -> fmt::Result {
        write!(
            self.0,
            "{}({segment}, {}, {}, {}, {})",
            Verb::Sense,
            sense.kind(),
            sense.specificity(),
            sense.confidence(),
            sense.explicitness()
        )
    }

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> fmt::Result {
        write!(self.0, "{}({source}, {target}, {relation})", Verb::Link)
    }

    fn propose(self, fresh: Option<&Label>, refines: &NodeRef, facet: Facet, title: &Title) -> fmt::Result {
        write!(
            self.0,
            "{}({}, {}({refines}), {facet}, {})",
            Verb::Propose,
            Fresh(fresh),
            Refines::Refines,
            Quoted(title)
        )
    }

    fn supersede(self, old: &NodeRef, by: &NodeRef) -> fmt::Result {
        write!(self.0, "{}({old}, {}({by}))", Verb::Supersede, By::By)
    }

    fn converse(self, segment: &SegmentRef) -> fmt::Result {
        write!(self.0, "{}({segment})", Verb::Conversational)
    }
}

impl Briefing<'_> {
    const FORMS: [Form; 6] =
        [Form::Place, Form::Sense, Form::Link, Form::Propose, Form::Supersede, Form::Conversational];
    const ROLE: &'static str = "You structure a project owner's words into a design graph. Reply with directive lines only: one directive per line, with no prose, no numbering and no code fences.";
    const COVERAGE: &'static str = "Give every segment listed below at least one directive. Mark chatter conversational. Otherwise place the segment on the node it is about, proposing a new node that refines the closest listed node when none fits, and sense it. A segment is written exactly as listed, its message id, a dot and its index; a node is n and its number, or the label of a node proposed earlier in your reply.";
}

impl fmt::Display for Briefing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Self::ROLE)?;
        writeln!(formatter)?;
        Self::FORMS.iter().try_for_each(|form| writeln!(formatter, "{form}"))?;
        writeln!(formatter)?;
        writeln!(formatter, "span-relation: {}", Choices(SpanRelation::VARIANTS))?;
        writeln!(formatter, "segment-relation: {}", Choices(SegmentRelation::VARIANTS))?;
        writeln!(formatter, "kind: {}", Choices(SenseKind::VARIANTS))?;
        writeln!(formatter, "specificity: {}", Choices(Specificity::VARIANTS))?;
        writeln!(formatter, "confidence: {}", Choices(Confidence::VARIANTS))?;
        writeln!(formatter, "explicitness: {}", Choices(Explicitness::VARIANTS))?;
        writeln!(formatter, "facet: {}", Choices(Facet::VARIANTS))?;
        writeln!(formatter)?;
        writeln!(formatter, "{}", Self::COVERAGE)?;
        writeln!(formatter)?;
        writeln!(formatter, "Segments:")?;
        self.job.segments().iter().try_for_each(|said| writeln!(formatter, "{}", Listed(said)))?;
        writeln!(formatter)?;
        writeln!(formatter, "Nodes:")?;
        self.job.context().iter().try_for_each(|node| writeln!(formatter, "{}", Context(node)))?;
        write!(formatter, "{}", self.attempt)
    }
}

impl<Choice: fmt::Display> fmt::Display for Choices<Choice> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let separators = iter::once("").chain(iter::repeat(" | "));
        self.0.iter().zip(separators).try_for_each(|(choice, separator)| write!(formatter, "{separator}{choice}"))
    }
}

impl fmt::Display for Listed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.0.segment(), Quote::quoting(self.0.body().as_ref()))
    }
}

impl fmt::Display for Context<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.0.entry();
        write!(formatter, "{} {} {} {}", Id(self.0.identity()), entry.project(), entry.facet(), entry.stage())?;
        self.0.title().iter().try_for_each(|title| write!(formatter, " {}", Quoted(title)))
    }
}

impl Job {
    #[must_use]
    pub fn prompt(&self, attempt: &Attempt) -> Prompt {
        Prompt::from(&Briefing { job: self, attempt })
    }
}
