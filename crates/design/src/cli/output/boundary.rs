use std::fmt;
use std::str::FromStr;

use design_model::{
    AnyNode, Artifact, Condition, Entered, Facet, Kin, NodeEdge, NodeId, Prefix, Project, Reason, Role,
    SegmentRelation, Sense, SpanRelation, Stage, Standing, Title,
};
use design_store::{AppliedId, RelationName, ShapeName};
use file_ports::Offset;
use grammar::cited::Cited;
use grammar::output::field::{Bare, Id, Integer, Phrase, Quoted, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;
use newtype::{Arbitrary, Display, Newtype};
use stream::{Relation, Shape};
use structurer::{Directive, SegmentRef};
use transcript_ports::{Authorship, Body, Cursor, Instant, Kind, SessionId, Uuid};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count(usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    node: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    role: Role,
    entered: Entered,
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
        entered: Entered,
    },
    Role {
        role: Role,
        entered: Entered,
    },
    Placed {
        relation: SpanRelation,
        segment: SegmentRef,
        author: Authorship,
        sense: Option<Sense>,
        entered: Entered,
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
    Annotated {
        remark: Remark,
        standing: Standing,
        entered: Entered,
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
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
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
            role: node.casting().role(),
            entered: Entered::from(entry.provenance()),
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
        }
    }
}

impl Line {
    fn read(fields: &mut Fields) -> Result<Line, Unread> {
        match fields.field::<Word<Key>>()? {
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
                sense: fields
                    .optional::<Word<_>>()
                    .map(|kind| {
                        Ok::<Sense, Unread>(Sense::new(
                            kind,
                            fields.field::<Word<_>>()?,
                            fields.field::<Word<_>>()?,
                            fields.field::<Word<_>>()?,
                        ))
                    })
                    .transpose()?,
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
            Key::Read | Key::To | Key::Project | Key::Entered | Key::At | Key::By | Key::File => Err(Unread::Unkeyed),
        }
    }

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
            role: fields.field::<Word<_>>()?,
            entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
            title: fields.optional::<Quoted<_>>(),
        })
    }

    fn rendered(&self) -> String {
        let Summary { node, project, facet, stage, role, entered, title } = self;
        let titled = title.as_ref().map(|title| format!(" {}", Line::quoted(title))).unwrap_or_default();
        format!(
            "{} {} {facet} {stage} {role} {} {entered}{titled}",
            Line::numbered(*node),
            Line::quoted(project),
            Key::Entered
        )
    }
}

impl Remark {
    fn read(fields: &mut Fields) -> Result<Remark, Unread> {
        match fields.field::<Word<Remarked>>()? {
            Remarked::Place => Ok(Remark::Place {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                relation: fields.field::<Word<_>>()?,
                node: fields.field::<Id<_>>()?,
            }),
            Remarked::Sense => Ok(Remark::Sense {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                sense: Sense::new(
                    fields.field::<Word<_>>()?,
                    fields.field::<Word<_>>()?,
                    fields.field::<Word<_>>()?,
                    fields.field::<Word<_>>()?,
                ),
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
