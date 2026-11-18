use std::fmt;
use std::str::FromStr;

use design_model::{
    Basis, Confidence, Explicitness, Facet, NodeId, Outline, Project, SegmentRelation, SenseKind, SpanRelation,
    Specificity, Stage, Title,
};
use grammar_output::field::{Id, Phrase, Quoted, Word};
use grammar_output::{Fields, Mapped, Parsed, Record, Unread};
use named_derive::Named;
use newtype::Enumerated;
use newtype::Wrapping;
use text_lines::Lines;

use crate::job::Listing;
use crate::lapse::Retried;
use crate::verb::Form;
use crate::{Attempt, Directive, Edge, Job, Label, Lapse, NodeRef, Proposal, Said};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Briefing(Vec<Brief>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Brief {
    Rule(Rule),
    Form(Form),
    Choices(Choice),
    Example(Directive),
    Said(Listing, Said),
    Node(Outline),
    Retry(Lapse),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Rule,
    Form,
    Choices,
    Example,
    Segment,
    Asked,
    Node,
    Retry,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Rule {
    #[name("You structure a project owner's words into a design graph.")]
    Role,
    #[name(
        "Reply with directive lines only, one directive per line, with no prose, numbering or code fences; each line takes one of the forms below."
    )]
    Reply,
    #[name("A choices line lists the words that one slot of the forms takes.")]
    Choose,
    #[name(
        "Sense as a question only a design question the owner has yet to decide; sense as an ask a request to the lead for status, explanation or analysis, which the lead's reply settles; sense an opinion, argument or instruction phrased as a question as an opinion, remark or instruction."
    )]
    Kinds,
    #[name(
        "Give every segment below at least one directive: mark chatter conversational; otherwise place it on the node it is about, proposing a node that refines the closest listed node when none fits, and sense it. A link is a directive for its first segment only, so a segment below that it links to needs its own; an asked line needs none. The listed nodes span every project; a node line names its project, and a segment goes under the project it is about."
    )]
    Cover,
    #[name(
        "Write a segment exactly as listed, its message id, a dot and its index; write a node as n and its number, or as the label of a node proposed earlier in your reply."
    )]
    Refer,
    #[name(
        "An asked line shows a question still open from earlier in the session; it is a link target only and is never given a directive of its own, whether place, sense or any other: when a segment answers or decides it, link that segment to it as answers, and place and sense that segment as usual."
    )]
    Link,
    #[name(
        "Your previous reply could not be used, for the reason on the retry line; reply again, following the forms exactly."
    )]
    Again,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sample {
    #[name("topic")]
    Label,
    #[name("what the segment is about")]
    Title,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
#[named(kebab)]
enum Choice {
    SpanRelation,
    SegmentRelation,
    Kind,
    Specificity,
    Confidence,
    Explicitness,
    Facet,
}

struct Choosing<'formatter, 'out>(&'out mut fmt::Formatter<'formatter>);

pub(crate) struct Quoting<'text, Text: fmt::Display>(&'text Text);

#[derive(Clone, Copy, Debug)]
struct Listed(Choice);

#[derive(Clone, Copy, Debug)]
pub struct Outlined<'outline>(&'outline Outline);

impl Rule {
    const OPENING: [Rule; 3] = [Rule::Role, Rule::Reply, Rule::Choose];
    const CLOSING: [Rule; 4] = [Rule::Kinds, Rule::Cover, Rule::Refer, Rule::Link];
}

impl Listing {
    const ALL: [Listing; 2] = [Listing::Segment, Listing::Asked];
}

impl From<Listing> for Key {
    fn from(listing: Listing) -> Key {
        match listing {
            Listing::Segment => Key::Segment,
            Listing::Asked => Key::Asked,
        }
    }
}

impl Form {
    const ALL: [Form; 6] = [Form::Place, Form::Sense, Form::Link, Form::Propose, Form::Supersede, Form::Conversational];
}

impl Choice {
    const ALL: [Choice; 7] = [
        Choice::SpanRelation,
        Choice::SegmentRelation,
        Choice::Kind,
        Choice::Specificity,
        Choice::Confidence,
        Choice::Explicitness,
        Choice::Facet,
    ];

    fn read(self, fields: Fields) -> Result<Parsed<Choice, Fields>, Unread> {
        self.passed(fields).map(|rest| Parsed::new(self, rest))
    }

    fn passed(self, fields: Fields) -> Result<Fields, Unread> {
        match self {
            Choice::SpanRelation => fields.past(SpanRelation::VARIANTS),
            Choice::SegmentRelation => fields.past(SegmentRelation::VARIANTS),
            Choice::Kind => fields.past(SenseKind::VARIANTS),
            Choice::Specificity => fields.past(Specificity::VARIANTS),
            Choice::Confidence => fields.past(Confidence::VARIANTS),
            Choice::Explicitness => fields.past(Explicitness::VARIANTS),
            Choice::Facet => fields.past(Facet::VARIANTS),
        }
    }

    fn listed(self, choosing: Choosing<'_, '_>) -> fmt::Result {
        match self {
            Choice::SpanRelation => choosing.words(SpanRelation::VARIANTS),
            Choice::SegmentRelation => choosing.words(SegmentRelation::VARIANTS),
            Choice::Kind => choosing.words(SenseKind::VARIANTS),
            Choice::Specificity => choosing.words(Specificity::VARIANTS),
            Choice::Confidence => choosing.words(Confidence::VARIANTS),
            Choice::Explicitness => choosing.words(Explicitness::VARIANTS),
            Choice::Facet => choosing.words(Facet::VARIANTS),
        }
    }
}

impl Choosing<'_, '_> {
    fn words<Word: fmt::Display>(self, words: &[Word]) -> fmt::Result {
        words.iter().try_for_each(|word| write!(self.0, " {word}"))
    }
}

impl Job {
    #[must_use]
    pub fn briefing(&self, attempt: &Attempt) -> Briefing {
        attempt.briefed(Briefing(
            Rule::OPENING
                .into_iter()
                .map(Brief::Rule)
                .chain(Form::ALL.into_iter().map(Brief::Form))
                .chain(Choice::ALL.into_iter().map(Brief::Choices))
                .chain(Rule::CLOSING.into_iter().map(Brief::Rule))
                .chain(Briefing::examples(self).into_iter().map(Brief::Example))
                .chain(
                    Listing::ALL.into_iter().flat_map(|listing| {
                        self.said(listing).iter().cloned().map(move |said| Brief::Said(listing, said))
                    }),
                )
                .chain(self.context().iter().cloned().map(Brief::Node))
                .collect(),
        ))
    }
}

impl Briefing {
    fn examples(job: &Job) -> Vec<Directive> {
        let segment = job.said(Listing::Segment).first().map(Said::segment);
        let node = job.context().first().map(|outline| NodeRef::Stored(outline.node()));
        let placed = segment.cloned().zip(node.clone()).map(|(said, outlined)| Directive::Place {
            segment: said,
            node: outlined,
            relation: SpanRelation::Mentions,
        });
        let label = Label::from(Sample::Label);
        let proposed = node.and_then(|refines| Edge::new(label.clone(), refines).ok()).map(|edge| Directive::Propose {
            proposal: Proposal::Labelled(edge),
            facet: Facet::Feature,
            title: Title::new(Sample::Title.to_string()),
        });
        let labelled = segment.cloned().zip(proposed).map(|(said, propose)| {
            [propose, Directive::Place { segment: said, node: NodeRef::from(label), relation: SpanRelation::Mentions }]
        });
        let conversational = segment.cloned().map(|said| Directive::Conversational { segment: said });
        placed.into_iter().chain(labelled.into_iter().flatten()).chain(conversational).collect()
    }

    #[must_use]
    pub fn retried(self, lapse: &Lapse) -> Briefing {
        Briefing(self.0.into_iter().chain([Brief::Retry(lapse.clone()), Brief::Rule(Rule::Again)]).collect())
    }
}

impl fmt::Display for Briefing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|brief| writeln!(formatter, "{brief}"))
    }
}

impl FromStr for Briefing {
    type Err = Unread;

    fn from_str(text: &str) -> Result<Briefing, Unread> {
        text.parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| line.parse())
            .collect::<Result<Vec<Brief>, Unread>>()
            .map(Briefing)
    }
}

impl fmt::Display for Brief {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Brief::Rule(rule) => write!(formatter, "{} {}", Key::Rule, Quoting::from(rule)),
            Brief::Form(form) => write!(formatter, "{} {}", Key::Form, Quoting::from(form)),
            Brief::Choices(choice) => write!(formatter, "{} {choice}{}", Key::Choices, Listed(*choice)),
            Brief::Example(directive) => write!(formatter, "{} {}", Key::Example, Quoting::from(directive)),
            Brief::Said(listing, said) => {
                write!(formatter, "{} {} {}", Key::from(*listing), said.segment(), Quoting::from(said.body()))
            }
            Brief::Node(outline) => write!(formatter, "{} {}", Key::Node, Outlined::from(outline)),
            Brief::Retry(lapse) => write!(formatter, "{} {}", Key::Retry, Retried::from(lapse)),
        }
    }
}

impl FromStr for Brief {
    type Err = Unread;

    fn from_str(text: &str) -> Result<Brief, Unread> {
        Fields::lexed(text)?.field::<Word<Key>>()?.concluded(Key::brief)?.ended()
    }
}

impl Key {
    fn brief(self, fields: Fields) -> Result<Parsed<Brief, Fields>, Unread> {
        match self {
            Key::Rule => fields.field::<Phrase<_>>().mapped(Brief::Rule),
            Key::Form => fields.field::<Phrase<_>>().mapped(Brief::Form),
            Key::Choices => fields.field::<Word<Choice>>()?.concluded(Choice::read).mapped(Brief::Choices),
            Key::Example => fields.field::<Phrase<_>>().mapped(Brief::Example),
            Key::Segment => fields.record::<Said>().mapped(|said| Brief::Said(Listing::Segment, said)),
            Key::Asked => fields.record::<Said>().mapped(|said| Brief::Said(Listing::Asked, said)),
            Key::Node => Outlined::read(fields).mapped(Brief::Node),
            Key::Retry => fields.record::<Lapse>().mapped(Brief::Retry),
        }
    }
}

impl Outlined<'_> {
    pub fn read(fields: Fields) -> Result<Parsed<Outline, Fields>, Unread> {
        fields.record::<OutlineLine>().mapped(Outline::from)
    }
}

#[derive(Debug, Record)]
struct OutlineLine {
    #[field(Id<_>)]
    node: NodeId,
    #[field(Phrase<_>)]
    project: Project,
    #[field(Word<_>)]
    facet: Facet,
    #[field(Word<_>)]
    stage: Stage,
    #[field(Word<_>)]
    basis: Basis,
    #[optional(Id<_>)]
    parent: Option<NodeId>,
    #[optional(Quoted<_>)]
    title: Option<Title>,
}

impl From<OutlineLine> for Outline {
    fn from(line: OutlineLine) -> Outline {
        Outline::new(line.node, line.project, line.facet, line.stage)
            .based(line.basis)
            .parented(line.parent)
            .titled(line.title)
    }
}

impl fmt::Display for Listed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.listed(Choosing(formatter))
    }
}

impl fmt::Display for Outlined<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let outline = self.0;
        write!(
            formatter,
            "n{} {} {} {} {}",
            outline.node().into_inner(),
            Quoting::from(outline.project()),
            outline.facet(),
            outline.stage(),
            outline.basis()
        )?;
        outline.parent().iter().try_for_each(|parent| write!(formatter, " n{}", parent.into_inner()))?;
        outline.title().iter().try_for_each(|title| write!(formatter, " {}", Quoting::from(*title)))
    }
}

impl<'outline> From<&'outline Outline> for Outlined<'outline> {
    fn from(outline: &'outline Outline) -> Outlined<'outline> {
        Outlined(outline)
    }
}

impl<'text, Text: fmt::Display> From<&'text Text> for Quoting<'text, Text> {
    fn from(text: &'text Text) -> Quoting<'text, Text> {
        Quoting(text)
    }
}

impl<Text: fmt::Display> fmt::Display for Quoting<'_, Text> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&Fields::quoting(&self.0.to_string()))
    }
}
