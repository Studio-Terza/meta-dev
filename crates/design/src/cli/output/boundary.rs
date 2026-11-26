use std::fmt;
use std::str::FromStr;

use design_model::{
    AnyNode, Artifact, Basis, Condition, Confidence, Explicitness, Facet, JobId, Kin, NodeEdge, NodeId, NodeRelation,
    Prefix, Project, Provenance, Reason, Role, RunId, SegmentRelation, Sense, SenseKind, SpanRelation, Specificity,
    Stage, Standing, Title,
};
use design_runs::{Excerpt, RunState};
use design_store::AppliedId;
use dispatch_ports::{Tokens, Usage};
use file_ports::Offset;
use grammar_cited::Cited;
use grammar_output::field::{Bare, Id, Integer, Phrase, Quoted, Word};
use grammar_output::{Fields, Mapped, Parsed, Record, Unread};
use newtype::{Listed, Wrapping};
use stream::{Relation, Shape};
use structurer::{Directive, Lapse, Retried, SegmentRef};
use strum::{Display, IntoStaticStr, VariantArray};
use transcript_ports::{Authorship, Body, Cursor, Instant, Kind, SessionId, Uuid};

use crate::cli::Viewed;
use crate::cli::count::Count;
use crate::cli::noted::{Note, Noted};
use crate::cli::viewed::Seen;

#[derive(Clone, Debug, PartialEq, Eq, Record)]
pub struct Summary {
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
    #[field(Word<_>)]
    role: Role,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
    #[optional(Quoted<_>)]
    title: Option<Title>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remark {
    Place { segment: SegmentRef, relation: SpanRelation, node: NodeId },
    Sense { segment: SegmentRef, sense: Sense },
    Link { source: SegmentRef, relation: SegmentRelation, target: SegmentRef },
    Conversational { segment: SegmentRef },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Anchored {
    Segment(SegmentRef),
    Message(Prefix),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Read {
        session: SessionId,
        cursor: Cursor,
    },
    Authored {
        author: Authorship,
        count: Count,
    },
    Unrecognised {
        kind: Kind,
        count: Count,
    },
    Formed {
        shape: Shape,
        count: Count,
    },
    Related {
        relation: Relation,
        count: Count,
    },
    Written(Count),
    Hit {
        segment: SegmentRef,
        author: Authorship,
        stamp: Instant,
        body: Body,
    },
    Root {
        node: NodeId,
        project: Project,
    },
    Consumes {
        node: NodeId,
        project: Project,
    },
    Node(Summary),
    Artifact(Artifact),
    Condition(Condition),
    Reason(Reason),
    Relative {
        kin: Kin,
        node: Summary,
    },
    Edge {
        edge: NodeEdge,
        entered: Provenance,
    },
    Role {
        role: Role,
        entered: Provenance,
    },
    Placed {
        relation: SpanRelation,
        segment: SegmentRef,
        author: Authorship,
        sense: Option<Sense>,
        entered: Provenance,
        body: Body,
    },
    Message {
        uuid: Uuid,
        author: Authorship,
        session: SessionId,
        stamp: Instant,
    },
    Segment {
        segment: SegmentRef,
        shape: Shape,
        body: Body,
    },
    Link {
        source: Anchored,
        relation: Relation,
        target: Anchored,
    },
    Refused(RefusedLink),
    Annotated {
        remark: Remark,
        standing: Standing,
        entered: Provenance,
    },
    Applied(Directive),
    Proposed {
        node: NodeId,
        directive: Directive,
    },
    Uncovered(SegmentRef),
    Repeated {
        file: AppliedId,
        stamp: Instant,
    },
    Reaped {
        run: RunId,
        state: RunState,
    },
    Interrupted(JobId),
    Ran(Attempted),
    Retried(Lapsed),
    Flailed(Lapsed),
    FlailedSegment(Flailing),
    Owed(Flailing),
    Viewed(Viewed),
    Noted(Noted),
}

#[derive(Clone, Debug, PartialEq, Eq, Record)]
pub struct RefusedLink {
    #[record]
    source: Anchored,
    #[field(Bare<_>)]
    relation: Relation,
    #[record]
    target: Anchored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempted {
    run: RunId,
    job: JobId,
    state: RunState,
    usage: Option<Usage>,
    reply: Option<Excerpt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Record)]
pub struct Flailing {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[past(Spend::Job)]
    #[field(Integer<_>, via = JobId::new)]
    job: JobId,
    #[past(Key::Run)]
    #[field(Integer<_>, via = RunId::new)]
    run: RunId,
}

#[derive(Clone, Debug, PartialEq, Eq, Record)]
pub struct Lapsed {
    #[field(Integer<_>, via = JobId::new)]
    job: JobId,
    #[field(Phrase<_>)]
    lapse: Lapse,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Key {
    Session,
    Read,
    To,
    Authorship,
    Unrecognised,
    Form,
    Relation,
    Written,
    Hit,
    Root,
    Consumes,
    Project,
    Node,
    Artifact,
    Condition,
    Reason,
    Ancestor,
    Child,
    Edge,
    Role,
    Entered,
    Placed,
    Message,
    At,
    Segment,
    Link,
    Refused,
    Annotation,
    Applied,
    Proposed,
    By,
    Uncovered,
    Already,
    File,
    Reaped,
    Interrupted,
    Run,
    Retried,
    Flailed,
    FlailedSegment,
    Owed,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Spend {
    Job,
    Tokens,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Replying {
    Reply,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Remarked {
    Place,
    Sense,
    Link,
    Conversational,
}

impl From<&AnyNode> for Summary {
    fn from(node: &AnyNode) -> Summary {
        let entry = node.entry();
        Summary {
            node: node.identity(),
            project: entry.project().clone(),
            facet: entry.facet(),
            stage: node.stage().stage(),
            basis: node.basis(),
            role: node.casting().role(),
            entered: entry.provenance(),
            title: node.title().cloned(),
        }
    }
}

impl FromStr for Line {
    type Err = Unread;

    fn from_str(text: &str) -> Result<Line, Unread> {
        Fields::lexed(text)?.record::<Line>()?.ended()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Read { session, cursor } => {
                write!(formatter, "{} {} {} {} {cursor}", Key::Session, Line::quoted(session), Key::Read, Key::To)
            }
            Line::Authored { author, count } => write!(formatter, "{} {author} {count}", Key::Authorship),
            Line::Unrecognised { kind, count } => {
                write!(formatter, "{} {} {count}", Key::Unrecognised, Line::quoted(kind))
            }
            Line::Formed { shape, count } => write!(formatter, "{} {shape} {count}", Key::Form),
            Line::Related { relation, count } => write!(formatter, "{} {relation} {count}", Key::Relation),
            Line::Written(count) => write!(formatter, "{} {count}", Key::Written),
            Line::Hit { segment, author, stamp, body } => {
                write!(formatter, "{} {segment} {author} {} {}", Key::Hit, Line::quoted(stamp), Line::quoted(body))
            }
            Line::Root { node, project } => {
                write!(formatter, "{} {} {} {}", Key::Root, Line::numbered(*node), Key::Project, Line::quoted(project))
            }
            Line::Consumes { node, project } => write!(
                formatter,
                "{} {} {} {}",
                Key::Consumes,
                Line::numbered(*node),
                Key::Project,
                Line::quoted(project)
            ),
            Line::Node(summary) => write!(formatter, "{} {}", Key::Node, summary.rendered()),
            Line::Artifact(artifact) => write!(formatter, "  {} {}", Key::Artifact, Line::quoted(artifact)),
            Line::Condition(condition) => write!(formatter, "  {} {}", Key::Condition, Line::quoted(condition)),
            Line::Reason(reason) => write!(formatter, "  {} {}", Key::Reason, Line::quoted(reason)),
            Line::Relative { kin, node } => write!(formatter, "  {kin} {}", node.rendered()),
            Line::Edge { edge, entered } => write!(
                formatter,
                "  {} {} {} {} {} {entered}",
                Key::Edge,
                Line::numbered(edge.ends()[0]),
                edge.relation(),
                Line::numbered(edge.ends()[1]),
                Key::Entered
            ),
            Line::Role { role, entered } => write!(formatter, "  {} {role} {} {entered}", Key::Role, Key::Entered),
            Line::Placed { relation, segment, author, sense, entered, body } => write!(
                formatter,
                "  {} {relation} {segment} {author}{} {} {entered} {}",
                Key::Placed,
                sense.map_or_default(Line::sensed),
                Key::Entered,
                Line::quoted(body)
            ),
            Line::Message { uuid, author, session, stamp } => write!(
                formatter,
                "{} {uuid} {author} {} {} {} {}",
                Key::Message,
                Key::Session,
                Line::quoted(session),
                Key::At,
                Line::quoted(stamp)
            ),
            Line::Segment { segment, shape, body } => {
                write!(formatter, "  {} {segment} {shape} {}", Key::Segment, Line::quoted(body))
            }
            Line::Link { source, relation, target } => {
                write!(formatter, "  {} {source} {relation} {target}", Key::Link)
            }
            Line::Refused(refused) => write!(formatter, "{} {} {refused}", Key::Refused, Key::Link),
            Line::Annotated { remark, standing, entered } => {
                write!(formatter, "  {} {} {standing} {} {entered}", Key::Annotation, remark.rendered(), Key::Entered)
            }
            Line::Applied(directive) => write!(formatter, "{} {}", Key::Applied, Line::quoted(directive)),
            Line::Proposed { node, directive } => {
                write!(formatter, "{} {} {} {}", Key::Proposed, Line::numbered(*node), Key::By, Line::quoted(directive))
            }
            Line::Uncovered(segment) => write!(formatter, "{} {segment}", Key::Uncovered),
            Line::Repeated { file, stamp } => write!(
                formatter,
                "{} {} {} {} {} {}",
                Key::Already,
                Key::Applied,
                Key::File,
                file.into_inner(),
                Key::At,
                Line::quoted(stamp)
            ),
            Line::Reaped { run, state } => write!(formatter, "{} {} {state}", Key::Reaped, run.into_inner()),
            Line::Interrupted(job) => write!(formatter, "{} {} {}", Key::Interrupted, Spend::Job, job.into_inner()),
            Line::Ran(attempted) => write!(formatter, "{} {attempted}", Key::Run),
            Line::Retried(lapsed) => write!(formatter, "{} {lapsed}", Key::Retried),
            Line::Flailed(lapsed) => write!(formatter, "{} {lapsed}", Key::Flailed),
            Line::FlailedSegment(flailing) => write!(formatter, "{} {flailing}", Key::FlailedSegment),
            Line::Owed(flailing) => write!(formatter, "{} {flailing}", Key::Owed),
            Line::Viewed(viewed) => write!(formatter, "{viewed}"),
            Line::Noted(noted) => write!(formatter, "{noted}"),
        }
    }
}

impl Record for Line {
    fn read(fields: Fields) -> Result<Parsed<Line, Fields>, Unread> {
        fields.optional::<Word<Seen>>().concluded(Seen::lined)
    }
}

impl Seen {
    fn lined(seen: Option<Seen>, fields: Fields) -> Result<Parsed<Line, Fields>, Unread> {
        match seen {
            Some(found) => found.viewed(fields).mapped(Line::Viewed),
            None => fields.optional::<Word<Note>>().concluded(Note::lined),
        }
    }
}

impl Note {
    fn lined(note: Option<Note>, fields: Fields) -> Result<Parsed<Line, Fields>, Unread> {
        match note {
            Some(found) => found.noted(fields).mapped(Line::Noted),
            None => fields.field::<Word<Key>>()?.concluded(Key::line),
        }
    }
}

impl Key {
    fn line(self, fields: Fields) -> Result<Parsed<Line, Fields>, Unread> {
        match self {
            Key::Session => fields.record::<Reading>().map(Parsed::converted),
            Key::Authorship => fields.record::<Authored>().map(Parsed::converted),
            Key::Unrecognised => fields.record::<Unrecognised>().map(Parsed::converted),
            Key::Form => fields.record::<Formed>().map(Parsed::converted),
            Key::Relation => fields.record::<Related>().map(Parsed::converted),
            Key::Written => fields.field::<Integer<_>>().mapped(|count| Line::Written(Count::new(count))),
            Key::Hit => fields.record::<Hit>().map(Parsed::converted),
            Key::Root => fields.record::<Rooted>().map(Parsed::converted),
            Key::Consumes => fields.record::<Consuming>().map(Parsed::converted),
            Key::Node => fields.record::<Summary>().mapped(Line::Node),
            Key::Artifact => fields.field::<Phrase<_>>().mapped(Line::Artifact),
            Key::Condition => fields.field::<Phrase<_>>().mapped(Line::Condition),
            Key::Reason => fields.field::<Phrase<_>>().mapped(Line::Reason),
            Key::Ancestor => fields.record::<Summary>().mapped(|node| Line::Relative { kin: Kin::Ancestor, node }),
            Key::Child => fields.record::<Summary>().mapped(|node| Line::Relative { kin: Kin::Child, node }),
            Key::Edge => fields.record::<Edged>().map(Parsed::converted),
            Key::Role => fields.record::<Cast>().map(Parsed::converted),
            Key::Placed => fields.record::<Placed>().map(Parsed::converted),
            Key::Message => fields.record::<Messaged>().map(Parsed::converted),
            Key::Segment => fields.record::<Segmented>().map(Parsed::converted),
            Key::Link => fields.record::<RefusedLink>().map(Parsed::converted),
            Key::Refused => fields.past(&[Key::Link])?.record::<RefusedLink>().mapped(Line::Refused),
            Key::Annotation => fields.record::<Annotated>().map(Parsed::converted),
            Key::Applied => fields.field::<Phrase<_>>().mapped(Line::Applied),
            Key::Proposed => fields.record::<Proposed>().map(Parsed::converted),
            Key::Uncovered => fields.field::<Cited>().mapped(|cited| Line::Uncovered(SegmentRef::from(cited))),
            Key::Already => fields.record::<Repeated>().map(Parsed::converted),
            Key::Reaped => fields.record::<Reaped>().map(Parsed::converted),
            Key::Interrupted => fields.record::<Interrupted>().map(Parsed::converted),
            Key::Run => fields.record::<Attempt>().mapped(|attempt| Line::Ran(Attempted::from(attempt))),
            Key::Retried => fields.record::<Lapsed>().mapped(Line::Retried),
            Key::Flailed => fields.record::<Lapsed>().mapped(Line::Flailed),
            Key::FlailedSegment => fields.record::<Flailing>().mapped(Line::FlailedSegment),
            Key::Owed => fields.record::<Flailing>().mapped(Line::Owed),
            Key::Read | Key::To | Key::Project | Key::Entered | Key::At | Key::By | Key::File => Err(Unread::Unkeyed),
        }
    }
}

impl Line {
    fn quoted(text: &impl fmt::Display) -> String {
        Fields::quoting(&text.to_string())
    }

    fn numbered(node: NodeId) -> String {
        format!("n{}", node.into_inner())
    }

    fn sensed(sense: Sense) -> String {
        format!(" {} {} {} {}", sense.kind(), sense.specificity(), sense.confidence(), sense.explicitness())
    }
}

impl fmt::Display for Anchored {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anchored::Segment(segment) => write!(formatter, "{segment}"),
            Anchored::Message(prefix) => write!(formatter, "{prefix}"),
        }
    }
}

impl RefusedLink {
    #[must_use]
    pub fn new(source: Anchored, relation: Relation, target: Anchored) -> RefusedLink {
        RefusedLink { source, relation, target }
    }
}

impl fmt::Display for RefusedLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.source, self.relation, self.target)
    }
}

impl Record for Anchored {
    fn read(fields: Fields) -> Result<Parsed<Anchored, Fields>, Unread> {
        fields.optional::<Cited>().concluded(Anchored::cited)
    }
}

impl Anchored {
    fn cited(cited: Option<Cited>, fields: Fields) -> Result<Parsed<Anchored, Fields>, Unread> {
        match cited {
            Some(found) => Ok(Parsed::new(Anchored::Segment(SegmentRef::from(found)), fields)),
            None => fields.field::<Bare<_>>().mapped(Anchored::Message),
        }
    }
}

impl Record for Sensed {
    fn read(fields: Fields) -> Result<Parsed<Sensed, Fields>, Unread> {
        fields.optional::<Word<SenseKind>>().concluded(Sensed::kinded)
    }
}

impl Sensed {
    fn kinded(kind: Option<SenseKind>, fields: Fields) -> Result<Parsed<Sensed, Fields>, Unread> {
        match kind {
            Some(found) => fields.record::<Graded>().mapped(|graded| {
                Sensed(Some(Sense::new(found, graded.specificity, graded.confidence, graded.explicitness)))
            }),
            None => Ok(Parsed::new(Sensed(None), fields)),
        }
    }
}

impl Record for Used {
    fn read(fields: Fields) -> Result<Parsed<Used, Fields>, Unread> {
        fields.optional::<Word<Spend>>().concluded(Used::spent)
    }
}

impl Used {
    fn spent(spend: Option<Spend>, fields: Fields) -> Result<Parsed<Used, Fields>, Unread> {
        match spend {
            Some(found) => found.used(fields),
            None => Ok(Parsed::new(Used(None), fields)),
        }
    }
}

impl Spend {
    fn used(self, fields: Fields) -> Result<Parsed<Used, Fields>, Unread> {
        match self {
            Spend::Tokens => fields.record::<Spent>().mapped(|spent| Used(Some(Usage::from(spent)))),
            Spend::Job => Err(Unread::Unkeyed),
        }
    }
}

impl Record for Replied {
    fn read(fields: Fields) -> Result<Parsed<Replied, Fields>, Unread> {
        fields.optional::<Word<Replying>>().concluded(Replied::replying)
    }
}

impl Replied {
    fn replying(replying: Option<Replying>, fields: Fields) -> Result<Parsed<Replied, Fields>, Unread> {
        match replying {
            Some(Replying::Reply) => fields.field::<Quoted<_>>().mapped(|reply| Replied(Some(reply))),
            None => Ok(Parsed::new(Replied(None), fields)),
        }
    }
}

impl Summary {
    fn rendered(&self) -> String {
        let Summary { node, project, facet, stage, basis, role, entered, title } = self;
        let titled = title.as_ref().map_or_default(|named| format!(" {}", Line::quoted(named)));
        format!(
            "{} {} {facet} {stage} {basis} {role} {} {entered}{titled}",
            Line::numbered(*node),
            Line::quoted(project),
            Key::Entered
        )
    }
}

impl Record for Remark {
    fn read(fields: Fields) -> Result<Parsed<Remark, Fields>, Unread> {
        fields.field::<Word<Remarked>>()?.concluded(Remarked::remark)
    }
}

impl Remarked {
    fn remark(self, fields: Fields) -> Result<Parsed<Remark, Fields>, Unread> {
        match self {
            Remarked::Place => fields.record::<Placing>().map(Parsed::converted),
            Remarked::Sense => fields.record::<Sensing>().map(Parsed::converted),
            Remarked::Link => fields.record::<Relating>().map(Parsed::converted),
            Remarked::Conversational => fields.record::<Conversing>().map(Parsed::converted),
        }
    }
}

impl Remark {
    fn rendered(&self) -> String {
        match self {
            Remark::Place { segment, relation, node } => {
                format!("{} {segment} {relation} {}", Remarked::Place, Line::numbered(*node))
            }
            Remark::Sense { segment, sense } => format!("{} {segment}{}", Remarked::Sense, Line::sensed(*sense)),
            Remark::Link { source, relation, target } => format!("{} {source} {relation} {target}", Remarked::Link),
            Remark::Conversational { segment } => format!("{} {segment}", Remarked::Conversational),
        }
    }
}

impl Attempted {
    #[must_use]
    pub fn new(run: RunId, job: JobId, state: RunState, usage: Option<Usage>) -> Attempted {
        Attempted { run, job, state, usage, reply: None }
    }

    #[must_use]
    pub fn replied(self, reply: Option<Excerpt>) -> Attempted {
        Attempted { reply, ..self }
    }
}

impl fmt::Display for Attempted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {} {}", self.run.into_inner(), Spend::Job, self.job.into_inner(), self.state)?;
        self.usage.iter().try_for_each(|usage| {
            let [input, output, written, read] = usage.spent();
            write!(formatter, " {} {input} {output} {written} {read}", Spend::Tokens)
        })?;
        self.reply.iter().try_for_each(|reply| write!(formatter, " {} {}", Replying::Reply, Line::quoted(reply)))
    }
}

impl Flailing {
    #[must_use]
    pub fn new(segment: SegmentRef, job: JobId, run: RunId) -> Flailing {
        Flailing { segment, job, run }
    }
}

impl fmt::Display for Flailing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} {} {} {}",
            self.segment,
            Spend::Job,
            self.job.into_inner(),
            Key::Run,
            self.run.into_inner()
        )
    }
}

impl Lapsed {
    #[must_use]
    pub fn new(job: JobId, lapse: Lapse) -> Lapsed {
        Lapsed { job, lapse }
    }
}

impl fmt::Display for Lapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.job.into_inner(), Line::quoted(&Retried::from(&self.lapse)))
    }
}

#[derive(Debug, Record)]
struct Reading {
    #[field(Quoted<_>)]
    session: SessionId,
    #[past(Key::Read, Key::To)]
    #[field(Bare<Offset>, via = Cursor::new)]
    cursor: Cursor,
}

#[derive(Debug, Record)]
struct Authored {
    #[field(Word<_>)]
    author: Authorship,
    #[field(Integer<_>, via = Count::new)]
    count: Count,
}

#[derive(Debug, Record)]
struct Unrecognised {
    #[field(Quoted<_>)]
    kind: Kind,
    #[field(Integer<_>, via = Count::new)]
    count: Count,
}

#[derive(Debug, Record)]
struct Formed {
    #[field(Word<_>)]
    shape: Shape,
    #[field(Integer<_>, via = Count::new)]
    count: Count,
}

#[derive(Debug, Record)]
struct Related {
    #[field(Word<_>)]
    relation: Relation,
    #[field(Integer<_>, via = Count::new)]
    count: Count,
}

#[derive(Debug, Record)]
struct Hit {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Word<_>)]
    author: Authorship,
    #[field(Quoted<_>)]
    stamp: Instant,
    #[field(Quoted<_>)]
    body: Body,
}

#[derive(Debug, Record)]
struct Rooted {
    #[field(Id<_>)]
    node: NodeId,
    #[past(Key::Project)]
    #[field(Phrase<_>)]
    project: Project,
}

#[derive(Debug, Record)]
struct Consuming {
    #[field(Id<_>)]
    node: NodeId,
    #[past(Key::Project)]
    #[field(Phrase<_>)]
    project: Project,
}

#[derive(Debug, Record)]
struct Edged {
    #[field(Id<_>)]
    source: NodeId,
    #[field(Word<_>)]
    relation: NodeRelation,
    #[field(Id<_>)]
    target: NodeId,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
}

#[derive(Debug, Record)]
struct Cast {
    #[field(Word<_>)]
    role: Role,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
}

#[derive(Debug, Record)]
struct Graded {
    #[field(Word<_>)]
    specificity: Specificity,
    #[field(Word<_>)]
    confidence: Confidence,
    #[field(Word<_>)]
    explicitness: Explicitness,
}

#[derive(Debug)]
struct Sensed(Option<Sense>);

#[derive(Debug, Record)]
struct Placed {
    #[field(Word<_>)]
    relation: SpanRelation,
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Word<_>)]
    author: Authorship,
    #[record]
    sense: Sensed,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
    #[field(Quoted<_>)]
    body: Body,
}

#[derive(Debug, Record)]
struct Messaged {
    #[field(Bare<_>)]
    uuid: Uuid,
    #[field(Word<_>)]
    author: Authorship,
    #[past(Key::Session)]
    #[field(Quoted<_>)]
    session: SessionId,
    #[past(Key::At)]
    #[field(Quoted<_>)]
    stamp: Instant,
}

#[derive(Debug, Record)]
struct Segmented {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Bare<_>)]
    shape: Shape,
    #[field(Quoted<_>)]
    body: Body,
}

#[derive(Debug, Record)]
struct Annotated {
    #[record]
    remark: Remark,
    #[field(Word<_>)]
    standing: Standing,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
}

#[derive(Debug, Record)]
struct Proposed {
    #[field(Id<_>)]
    node: NodeId,
    #[past(Key::By)]
    #[field(Phrase<_>)]
    directive: Directive,
}

#[derive(Debug, Record)]
struct Repeated {
    #[past(Key::Applied, Key::File)]
    #[field(Integer<_>, via = AppliedId::new)]
    file: AppliedId,
    #[past(Key::At)]
    #[field(Quoted<_>)]
    stamp: Instant,
}

#[derive(Debug, Record)]
struct Reaped {
    #[field(Integer<_>, via = RunId::new)]
    run: RunId,
    #[field(Word<_>)]
    state: RunState,
}

#[derive(Debug, Record)]
struct Interrupted {
    #[past(Spend::Job)]
    #[field(Integer<_>, via = JobId::new)]
    job: JobId,
}

#[derive(Debug, Record)]
struct Placing {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Word<_>)]
    relation: SpanRelation,
    #[field(Id<_>)]
    node: NodeId,
}

#[derive(Debug, Record)]
struct Sensing {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Word<_>)]
    kind: SenseKind,
    #[record]
    graded: Graded,
}

#[derive(Debug, Record)]
struct Relating {
    #[field(Cited, via = SegmentRef::from)]
    source: SegmentRef,
    #[field(Word<_>)]
    relation: SegmentRelation,
    #[field(Cited, via = SegmentRef::from)]
    target: SegmentRef,
}

#[derive(Debug, Record)]
struct Conversing {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
}

#[derive(Debug, Record)]
struct Spent {
    #[field(Integer<_>, via = Tokens::new)]
    input: Tokens,
    #[field(Integer<_>, via = Tokens::new)]
    output: Tokens,
    #[field(Integer<_>, via = Tokens::new)]
    written: Tokens,
    #[field(Integer<_>, via = Tokens::new)]
    read: Tokens,
}

#[derive(Debug, Record)]
struct Attempt {
    #[field(Integer<_>, via = RunId::new)]
    run: RunId,
    #[past(Spend::Job)]
    #[field(Integer<_>, via = JobId::new)]
    job: JobId,
    #[field(Word<_>)]
    state: RunState,
    #[record]
    usage: Used,
    #[record]
    reply: Replied,
}

#[derive(Debug)]
struct Used(Option<Usage>);

#[derive(Debug)]
struct Replied(Option<Excerpt>);

impl From<Spent> for Usage {
    fn from(spent: Spent) -> Usage {
        Usage::new(spent.input, spent.output, spent.written, spent.read)
    }
}

impl From<Attempt> for Attempted {
    fn from(read: Attempt) -> Attempted {
        Attempted { run: read.run, job: read.job, state: read.state, usage: read.usage.0, reply: read.reply.0 }
    }
}

impl From<Reading> for Line {
    fn from(read: Reading) -> Line {
        Line::Read { session: read.session, cursor: read.cursor }
    }
}

impl From<Authored> for Line {
    fn from(read: Authored) -> Line {
        Line::Authored { author: read.author, count: read.count }
    }
}

impl From<Unrecognised> for Line {
    fn from(read: Unrecognised) -> Line {
        Line::Unrecognised { kind: read.kind, count: read.count }
    }
}

impl From<Formed> for Line {
    fn from(read: Formed) -> Line {
        Line::Formed { shape: read.shape, count: read.count }
    }
}

impl From<Related> for Line {
    fn from(read: Related) -> Line {
        Line::Related { relation: read.relation, count: read.count }
    }
}

impl From<Hit> for Line {
    fn from(read: Hit) -> Line {
        Line::Hit { segment: read.segment, author: read.author, stamp: read.stamp, body: read.body }
    }
}

impl From<Rooted> for Line {
    fn from(read: Rooted) -> Line {
        Line::Root { node: read.node, project: read.project }
    }
}

impl From<Consuming> for Line {
    fn from(read: Consuming) -> Line {
        Line::Consumes { node: read.node, project: read.project }
    }
}

impl From<Edged> for Line {
    fn from(read: Edged) -> Line {
        Line::Edge { edge: NodeEdge::new(read.source, read.relation, read.target), entered: read.entered }
    }
}

impl From<Cast> for Line {
    fn from(read: Cast) -> Line {
        Line::Role { role: read.role, entered: read.entered }
    }
}

impl From<Placed> for Line {
    fn from(read: Placed) -> Line {
        let Placed { relation, segment, author, sense, entered, body } = read;
        Line::Placed { relation, segment, author, sense: sense.0, entered, body }
    }
}

impl From<Messaged> for Line {
    fn from(read: Messaged) -> Line {
        Line::Message { uuid: read.uuid, author: read.author, session: read.session, stamp: read.stamp }
    }
}

impl From<Segmented> for Line {
    fn from(read: Segmented) -> Line {
        Line::Segment { segment: read.segment, shape: read.shape, body: read.body }
    }
}

impl From<RefusedLink> for Line {
    fn from(read: RefusedLink) -> Line {
        Line::Link { source: read.source, relation: read.relation, target: read.target }
    }
}

impl From<Annotated> for Line {
    fn from(read: Annotated) -> Line {
        Line::Annotated { remark: read.remark, standing: read.standing, entered: read.entered }
    }
}

impl From<Proposed> for Line {
    fn from(read: Proposed) -> Line {
        Line::Proposed { node: read.node, directive: read.directive }
    }
}

impl From<Repeated> for Line {
    fn from(read: Repeated) -> Line {
        Line::Repeated { file: read.file, stamp: read.stamp }
    }
}

impl From<Reaped> for Line {
    fn from(read: Reaped) -> Line {
        Line::Reaped { run: read.run, state: read.state }
    }
}

impl From<Interrupted> for Line {
    fn from(read: Interrupted) -> Line {
        Line::Interrupted(read.job)
    }
}

impl From<Placing> for Remark {
    fn from(read: Placing) -> Remark {
        Remark::Place { segment: read.segment, relation: read.relation, node: read.node }
    }
}

impl From<Sensing> for Remark {
    fn from(read: Sensing) -> Remark {
        let Graded { specificity, confidence, explicitness } = read.graded;
        Remark::Sense { segment: read.segment, sense: Sense::new(read.kind, specificity, confidence, explicitness) }
    }
}

impl From<Relating> for Remark {
    fn from(read: Relating) -> Remark {
        Remark::Link { source: read.source, relation: read.relation, target: read.target }
    }
}

impl From<Conversing> for Remark {
    fn from(read: Conversing) -> Remark {
        Remark::Conversational { segment: read.segment }
    }
}
