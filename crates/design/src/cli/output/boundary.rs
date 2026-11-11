use std::fmt;
use std::str::FromStr;

use design_model::{
    AnyNode, Artifact, Basis, Condition, Facet, JobId, Kin, NodeEdge, NodeId, Prefix, Project, Provenance, Reason,
    Role, RunId, SegmentRelation, Sense, SenseKind, SpanRelation, Stage, Standing, Title,
};
use design_runs::{Excerpt, RunState};
use design_store::{AppliedId, RelationName, ShapeName};
use dispatch_ports::{Tokens, Usage};
use file_ports::Offset;
use grammar_cited::Cited;
use grammar_output::field::{Bare, Id, Integer, Phrase, Quoted, Word};
use grammar_output::{Fields, Unread};
use named_derive::Named;
use newtype::Wrapping;
use stream::{Relation, Shape};
use structurer::{Directive, Lapse, Retried, SegmentRef};
use transcript_ports::{Authorship, Body, Cursor, Instant, Kind, SessionId, Uuid};

use crate::cli::Viewed;
use crate::cli::count::Count;
use crate::cli::noted::{Note, Noted};
use crate::cli::viewed::Seen;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    node: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    basis: Basis,
    role: Role,
    entered: Provenance,
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
        shape: ShapeName,
        body: Body,
    },
    Link {
        source: Anchored,
        relation: RelationName,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedLink {
    source: Anchored,
    relation: RelationName,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flailing {
    segment: SegmentRef,
    job: JobId,
    run: RunId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lapsed {
    job: JobId,
    lapse: Lapse,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    #[name("session")]
    Session,
    #[name("read")]
    Read,
    #[name("to")]
    To,
    #[name("authorship")]
    Authorship,
    #[name("unrecognised")]
    Unrecognised,
    #[name("form")]
    Form,
    #[name("relation")]
    Relation,
    #[name("written")]
    Written,
    #[name("hit")]
    Hit,
    #[name("root")]
    Root,
    #[name("consumes")]
    Consumes,
    #[name("project")]
    Project,
    #[name("node")]
    Node,
    #[name("artifact")]
    Artifact,
    #[name("condition")]
    Condition,
    #[name("reason")]
    Reason,
    #[name("ancestor")]
    Ancestor,
    #[name("child")]
    Child,
    #[name("edge")]
    Edge,
    #[name("role")]
    Role,
    #[name("entered")]
    Entered,
    #[name("placed")]
    Placed,
    #[name("message")]
    Message,
    #[name("at")]
    At,
    #[name("segment")]
    Segment,
    #[name("link")]
    Link,
    #[name("refused")]
    Refused,
    #[name("annotation")]
    Annotation,
    #[name("applied")]
    Applied,
    #[name("proposed")]
    Proposed,
    #[name("by")]
    By,
    #[name("uncovered")]
    Uncovered,
    #[name("already")]
    Already,
    #[name("file")]
    File,
    #[name("reaped")]
    Reaped,
    #[name("interrupted")]
    Interrupted,
    #[name("run")]
    Run,
    #[name("retried")]
    Retried,
    #[name("flailed")]
    Flailed,
    #[name("flailed-segment")]
    FlailedSegment,
    #[name("owed")]
    Owed,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Spend {
    #[name("job")]
    Job,
    #[name("tokens")]
    Tokens,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Replying {
    #[name("reply")]
    Reply,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Remarked {
    #[name("place")]
    Place,
    #[name("sense")]
    Sense,
    #[name("link")]
    Link,
    #[name("conversational")]
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
        let mut fields = Fields::lexed(text)?;
        let line = Line::read(&mut fields)?;
        fields.ended(line)
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
                sense.map(Line::sensed).unwrap_or_default(),
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

impl Line {
    fn read(fields: &mut Fields) -> Result<Line, Unread> {
        fields
            .optional::<Word<Seen>>()
            .map(|seen| Viewed::read(seen, fields).map(Line::Viewed))
            .or_else(|| fields.optional::<Word<Note>>().map(|note| Noted::read(note, fields).map(Line::Noted)))
            .unwrap_or_else(|| fields.field::<Word<Key>>().and_then(|key| key.line(fields)))
    }
}

impl Key {
    fn line(self, fields: &mut Fields) -> Result<Line, Unread> {
        match self {
            Key::Session => Ok(Line::Read {
                session: fields.field::<Quoted<_>>()?,
                cursor: Cursor::new(Offset::new(fields.past(&[Key::Read, Key::To])?.field::<Integer<_>>()?)),
            }),
            Key::Authorship => Ok(Line::Authored {
                author: fields.field::<Word<_>>()?,
                count: Count::new(fields.field::<Integer<_>>()?),
            }),
            Key::Unrecognised => Ok(Line::Unrecognised {
                kind: fields.field::<Quoted<_>>()?,
                count: Count::new(fields.field::<Integer<_>>()?),
            }),
            Key::Form => {
                Ok(Line::Formed { shape: fields.field::<Word<_>>()?, count: Count::new(fields.field::<Integer<_>>()?) })
            }
            Key::Relation => Ok(Line::Related {
                relation: fields.field::<Word<_>>()?,
                count: Count::new(fields.field::<Integer<_>>()?),
            }),
            Key::Written => Ok(Line::Written(Count::new(fields.field::<Integer<_>>()?))),
            Key::Hit => Ok(Line::Hit {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                author: fields.field::<Word<_>>()?,
                stamp: fields.field::<Quoted<_>>()?,
                body: fields.field::<Quoted<_>>()?,
            }),
            Key::Root => Ok(Line::Root {
                node: fields.field::<Id<_>>()?,
                project: fields.past(&[Key::Project])?.field::<Phrase<_>>()?,
            }),
            Key::Consumes => Ok(Line::Consumes {
                node: fields.field::<Id<_>>()?,
                project: fields.past(&[Key::Project])?.field::<Phrase<_>>()?,
            }),
            Key::Node => Summary::read(fields).map(Line::Node),
            Key::Artifact => Ok(Line::Artifact(fields.field::<Phrase<_>>()?)),
            Key::Condition => Ok(Line::Condition(fields.field::<Phrase<_>>()?)),
            Key::Reason => Ok(Line::Reason(fields.field::<Phrase<_>>()?)),
            Key::Ancestor => Ok(Line::Relative { kin: Kin::Ancestor, node: Summary::read(fields)? }),
            Key::Child => Ok(Line::Relative { kin: Kin::Child, node: Summary::read(fields)? }),
            Key::Edge => Ok(Line::Edge {
                edge: NodeEdge::new(fields.field::<Id<_>>()?, fields.field::<Word<_>>()?, fields.field::<Id<_>>()?),
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            }),
            Key::Role => Ok(Line::Role {
                role: fields.field::<Word<_>>()?,
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            }),
            Key::Placed => Ok(Line::Placed {
                relation: fields.field::<Word<_>>()?,
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                author: fields.field::<Word<_>>()?,
                sense: fields.optional::<Word<_>>().map(|kind| Remark::sensed(kind, fields)).transpose()?,
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
                body: fields.field::<Quoted<_>>()?,
            }),
            Key::Message => Ok(Line::Message {
                uuid: fields.field::<Bare<_>>()?,
                author: fields.field::<Word<_>>()?,
                session: fields.past(&[Key::Session])?.field::<Quoted<_>>()?,
                stamp: fields.past(&[Key::At])?.field::<Quoted<_>>()?,
            }),
            Key::Segment => Ok(Line::Segment {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                shape: fields.field::<Bare<_>>()?,
                body: fields.field::<Quoted<_>>()?,
            }),
            Key::Link => Ok(Line::Link {
                source: Anchored::read(fields)?,
                relation: fields.field::<Bare<_>>()?,
                target: Anchored::read(fields)?,
            }),
            Key::Refused => RefusedLink::read(fields.past(&[Key::Link])?).map(Line::Refused),
            Key::Annotation => Ok(Line::Annotated {
                remark: Remark::read(fields)?,
                standing: fields.field::<Word<_>>()?,
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            }),
            Key::Applied => Ok(Line::Applied(fields.field::<Phrase<_>>()?)),
            Key::Proposed => Ok(Line::Proposed {
                node: fields.field::<Id<_>>()?,
                directive: fields.past(&[Key::By])?.field::<Phrase<_>>()?,
            }),
            Key::Uncovered => Ok(Line::Uncovered(SegmentRef::from(&fields.field::<Cited>()?))),
            Key::Already => Ok(Line::Repeated {
                file: AppliedId::new(fields.past(&[Key::Applied, Key::File])?.field::<Integer<_>>()?),
                stamp: fields.past(&[Key::At])?.field::<Quoted<_>>()?,
            }),
            Key::Reaped => {
                Ok(Line::Reaped { run: RunId::new(fields.field::<Integer<_>>()?), state: fields.field::<Word<_>>()? })
            }
            Key::Interrupted => Ok(Line::Interrupted(JobId::new(fields.past(&[Spend::Job])?.field::<Integer<_>>()?))),
            Key::Run => Attempted::read(fields).map(Line::Ran),
            Key::Retried => Lapsed::read(fields).map(Line::Retried),
            Key::Flailed => Lapsed::read(fields).map(Line::Flailed),
            Key::FlailedSegment => Flailing::read(fields).map(Line::FlailedSegment),
            Key::Owed => Flailing::read(fields).map(Line::Owed),
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
    pub fn new(source: Anchored, relation: RelationName, target: Anchored) -> RefusedLink {
        RefusedLink { source, relation, target }
    }

    fn read(fields: &mut Fields) -> Result<RefusedLink, Unread> {
        Ok(RefusedLink {
            source: Anchored::read(fields)?,
            relation: fields.field::<Bare<_>>()?,
            target: Anchored::read(fields)?,
        })
    }
}

impl fmt::Display for RefusedLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.source, self.relation, self.target)
    }
}

impl Anchored {
    fn read(fields: &mut Fields) -> Result<Anchored, Unread> {
        fields.optional::<Cited>().map_or_else(
            || fields.field::<Bare<_>>().map(Anchored::Message),
            |cited| Ok(Anchored::Segment(SegmentRef::from(&cited))),
        )
    }
}

impl Summary {
    fn read(fields: &mut Fields) -> Result<Summary, Unread> {
        Ok(Summary {
            node: fields.field::<Id<_>>()?,
            project: fields.field::<Phrase<_>>()?,
            facet: fields.field::<Word<_>>()?,
            stage: fields.field::<Word<_>>()?,
            basis: fields.field::<Word<_>>()?,
            role: fields.field::<Word<_>>()?,
            entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            title: fields.optional::<Quoted<_>>(),
        })
    }

    fn rendered(&self) -> String {
        let Summary { node, project, facet, stage, basis, role, entered, title } = self;
        let titled = title.as_ref().map(|named| format!(" {}", Line::quoted(named))).unwrap_or_default();
        format!(
            "{} {} {facet} {stage} {basis} {role} {} {entered}{titled}",
            Line::numbered(*node),
            Line::quoted(project),
            Key::Entered
        )
    }
}

impl Remark {
    fn sensed(kind: SenseKind, fields: &mut Fields) -> Result<Sense, Unread> {
        Ok(Sense::new(kind, fields.field::<Word<_>>()?, fields.field::<Word<_>>()?, fields.field::<Word<_>>()?))
    }

    fn read(fields: &mut Fields) -> Result<Remark, Unread> {
        match fields.field::<Word<Remarked>>()? {
            Remarked::Place => Ok(Remark::Place {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                relation: fields.field::<Word<_>>()?,
                node: fields.field::<Id<_>>()?,
            }),
            Remarked::Sense => Ok(Remark::Sense {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                sense: Remark::sensed(fields.field::<Word<_>>()?, fields)?,
            }),
            Remarked::Link => Ok(Remark::Link {
                source: SegmentRef::from(&fields.field::<Cited>()?),
                relation: fields.field::<Word<_>>()?,
                target: SegmentRef::from(&fields.field::<Cited>()?),
            }),
            Remarked::Conversational => {
                Ok(Remark::Conversational { segment: SegmentRef::from(&fields.field::<Cited>()?) })
            }
        }
    }

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

    fn read(fields: &mut Fields) -> Result<Attempted, Unread> {
        Ok(Attempted {
            run: RunId::new(fields.field::<Integer<_>>()?),
            job: JobId::new(fields.past(&[Spend::Job])?.field::<Integer<_>>()?),
            state: fields.field::<Word<_>>()?,
            usage: Attempted::usage(fields.optional::<Word<Spend>>(), fields)?,
            reply: fields
                .optional::<Word<Replying>>()
                .map(|Replying::Reply| fields.field::<Quoted<_>>())
                .transpose()?,
        })
    }

    fn usage(key: Option<Spend>, fields: &mut Fields) -> Result<Option<Usage>, Unread> {
        match key {
            None => Ok(None),
            Some(Spend::Tokens) => Attempted::tokens(fields).map(Some),
            Some(Spend::Job) => Err(Unread::Unkeyed),
        }
    }

    fn tokens(fields: &mut Fields) -> Result<Usage, Unread> {
        Ok(Usage::new(
            Tokens::new(fields.field::<Integer<_>>()?),
            Tokens::new(fields.field::<Integer<_>>()?),
            Tokens::new(fields.field::<Integer<_>>()?),
            Tokens::new(fields.field::<Integer<_>>()?),
        ))
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

    fn read(fields: &mut Fields) -> Result<Flailing, Unread> {
        Ok(Flailing {
            segment: SegmentRef::from(&fields.field::<Cited>()?),
            job: JobId::new(fields.past(&[Spend::Job])?.field::<Integer<_>>()?),
            run: RunId::new(fields.past(&[Key::Run])?.field::<Integer<_>>()?),
        })
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

    fn read(fields: &mut Fields) -> Result<Lapsed, Unread> {
        Ok(Lapsed { job: JobId::new(fields.field::<Integer<_>>()?), lapse: fields.field::<Phrase<_>>()? })
    }
}

impl fmt::Display for Lapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.job.into_inner(), Line::quoted(&Retried::from(&self.lapse)))
    }
}
