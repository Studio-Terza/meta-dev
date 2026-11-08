use std::fmt;
use std::str::FromStr;

use design_model::{Confidence, Explicitness, Facet, Outline, SegmentRelation, SenseKind, SpanRelation, Specificity};
use grammar::cited::Cited;
use grammar::output::field::{Id, Phrase, Quoted, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;
use newtype::Wrapping;
use strum::VariantArray;
use text_lines::Lines;

use crate::lapse::Retried;
use crate::verb::Form;
use crate::{Attempt, Directive, Job, Lapse, NodeRef, Said, SegmentRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Briefing(Vec<Brief>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Brief {
    Rule(Rule),
    Form(Form),
    Choices(Choice),
    Example(Directive),
    Segment(Said),
    Node(Outline),
    Retry(Lapse),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    #[name("rule")]
    Rule,
    #[name("form")]
    Form,
    #[name("choices")]
    Choices,
    #[name("example")]
    Example,
    #[name("segment")]
    Segment,
    #[name("node")]
    Node,
    #[name("retry")]
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
        "Give every segment below at least one directive: mark chatter conversational; otherwise place it on the node it is about, proposing a node that refines the closest listed node when none fits, and sense it. A link is a directive for its first segment only, so a segment it links to needs its own. The listed nodes span every project; a node line names its project, and a segment goes under the project it is about."
    )]
    Cover,
    #[name(
        "Write a segment exactly as listed, its message id, a dot and its index; write a node as n and its number, or as the label of a node proposed earlier in your reply."
    )]
    Refer,
    #[name(
        "Your previous reply could not be used, for the reason on the retry line; reply again, following the forms exactly."
    )]
    Again,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    #[name("span-relation")]
    SpanRelation,
    #[name("segment-relation")]
    SegmentRelation,
    #[name("kind")]
    Kind,
    #[name("specificity")]
    Specificity,
    #[name("confidence")]
    Confidence,
    #[name("explicitness")]
    Explicitness,
    #[name("facet")]
    Facet,
}

struct Choosing<'formatter, 'out>(&'out mut fmt::Formatter<'formatter>);

pub(crate) struct Quoting<'text, Text: fmt::Display>(&'text Text);

#[derive(Clone, Copy, Debug)]
struct Listed(Choice);

#[derive(Clone, Copy, Debug)]
struct Outlined<'outline>(&'outline Outline);

impl Rule {
    const OPENING: [Rule; 3] = [Rule::Role, Rule::Reply, Rule::Choose];
    const CLOSING: [Rule; 2] = [Rule::Cover, Rule::Refer];
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

    fn read(self, fields: &mut Fields) -> Result<Choice, Unread> {
        self.passed(fields).map(|_fields| self)
    }

    fn passed(self, fields: &mut Fields) -> Result<&mut Fields, Unread> {
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
                .chain(self.examples().into_iter().map(Brief::Example))
                .chain(self.segments().iter().cloned().map(Brief::Segment))
                .chain(self.context().iter().cloned().map(Brief::Node))
                .collect(),
        ))
    }

    fn examples(&self) -> Vec<Directive> {
        let segment = self.segments().first().map(Said::segment);
        let node = self.context().first().map(|outline| NodeRef::Stored(outline.node()));
        let placed = segment.cloned().zip(node).map(|(said, outlined)| Directive::Place {
            segment: said,
            node: outlined,
            relation: SpanRelation::Mentions,
        });
        let conversational = segment.cloned().map(|said| Directive::Conversational { segment: said });
        placed.into_iter().chain(conversational).collect()
    }
}

impl Briefing {
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
        Lines::from(text).map(str::parse).collect::<Result<Vec<Brief>, Unread>>().map(Briefing)
    }
}

impl fmt::Display for Brief {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Brief::Rule(rule) => write!(formatter, "{} {}", Key::Rule, Quoting::from(rule)),
            Brief::Form(form) => write!(formatter, "{} {}", Key::Form, Quoting::from(form)),
            Brief::Choices(choice) => write!(formatter, "{} {choice}{}", Key::Choices, Listed(*choice)),
            Brief::Example(directive) => write!(formatter, "{} {}", Key::Example, Quoting::from(directive)),
            Brief::Segment(said) => {
                write!(formatter, "{} {} {}", Key::Segment, said.segment(), Quoting::from(said.body()))
            }
            Brief::Node(outline) => write!(formatter, "{} {}", Key::Node, Outlined(outline)),
            Brief::Retry(lapse) => write!(formatter, "{} {}", Key::Retry, Retried::from(lapse)),
        }
    }
}

impl FromStr for Brief {
    type Err = Unread;

    fn from_str(text: &str) -> Result<Brief, Unread> {
        let mut fields = Fields::lexed(text)?;
        let brief = Brief::read(&mut fields)?;
        fields.ended(brief)
    }
}

impl Brief {
    fn read(fields: &mut Fields) -> Result<Brief, Unread> {
        match fields.field::<Word<Key>>()? {
            Key::Rule => Ok(Brief::Rule(fields.field::<Phrase<_>>()?)),
            Key::Form => Ok(Brief::Form(fields.field::<Phrase<_>>()?)),
            Key::Choices => fields.field::<Word<Choice>>()?.read(fields).map(Brief::Choices),
            Key::Example => Ok(Brief::Example(fields.field::<Phrase<_>>()?)),
            Key::Segment => Said::read(fields).map(Brief::Segment),
            Key::Node => Outlined::read(fields).map(Brief::Node),
            Key::Retry => Lapse::read(fields).map(Brief::Retry),
        }
    }
}

impl Said {
    fn read(fields: &mut Fields) -> Result<Said, Unread> {
        Ok(Said::new(SegmentRef::from(&fields.field::<Cited>()?), fields.field::<Quoted<_>>()?))
    }
}

impl Outlined<'_> {
    fn read(fields: &mut Fields) -> Result<Outline, Unread> {
        let outline = Outline::new(
            fields.field::<Id<_>>()?,
            fields.field::<Phrase<_>>()?,
            fields.field::<Word<_>>()?,
            fields.field::<Word<_>>()?,
        );
        Ok(outline.parented(fields.optional::<Id<_>>()).titled(fields.optional::<Quoted<_>>()))
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
            "n{} {} {} {}",
            outline.node().into_inner(),
            Quoting::from(outline.project()),
            outline.facet(),
            outline.stage()
        )?;
        outline.parent().iter().try_for_each(|parent| write!(formatter, " n{}", parent.into_inner()))?;
        outline.title().iter().try_for_each(|title| write!(formatter, " {}", Quoting::from(*title)))
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
