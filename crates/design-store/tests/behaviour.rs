use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use design_model::{
    Entered, Entry, Facet, JobId, Kin, Node, NodeId, NodeRelation, Ordinal, Prefix, Project, Provenance, RunId,
    Segment, SegmentRelation, Stage, Supersession, Title,
};
use design_store::{
    AppliedFile, ContextNodes, Covered, Cycle, Directed, Directions, JobClosed, JobOpened, JobState, Linked, LiveRuns,
    NewNode, NextJob, NextNode, NextRun, Placed, Prefixed, ProjectRoot, Refusal, RunEnded, RunSpawned, RunState, Scope,
    SearchSegments, Seen, SessionCursors, SessionRecords, ShownMessage, ShownNode, SpentOn, StreamSchema, Unseen,
    Unstructured,
};
use dispatch_ports::{Deadline, Pid, Spawned, Started, Tokens, Usage, Wall};
use file_ports::{Bytes, Files, Memory};
use grammar::search::Terms;
use laws::proptest::collection::vec;
use laws::proptest::prelude::{Just, any, prop_assert_eq};
use laws::proptest::sample::{Index, select};
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use newtype::{Counter, Newtype};
use port_core::{Port, Run};
use store_ports::{Sqlite, Store, Written};
use stream::Stream;
use strum::VariantArray;
use transcript_ports::{Authorship, Body, ClaudeCodeDir, Cursor, SessionId, Transcripts, Uuid};

const DIRECTORY: &str = "/projects/work";

const SESSION: &str = concat!(
    "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},\"message\":{\"content\":\"1. go\\n2. now\"}}\n",
    "{\"type\":\"assistant\",\"uuid\":\"l1\",\"parentUuid\":\"o1\",\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:01Z\",\"origin\":null,\"message\":{\"model\":\"claude\",\"content\":[{\"type\":\"text\",\"text\":\"done\"}]}}\n"
);

struct Recorded {
    records: SessionRecords,
    cursor: Cursor,
}

impl Recorded {
    fn session() -> Result<Recorded, Box<dyn Error>> {
        Recorded::of(SESSION)
    }

    fn of(lines: &str) -> Result<Recorded, Box<dyn Error>> {
        let memory = Memory::default();
        memory.create_directories(Path::new(DIRECTORY))?;
        memory.write(&Path::new(DIRECTORY).join("s.jsonl"), Bytes::from(lines))?;
        let run = Run::root();
        let port = run.port(ClaudeCodeDir::new(memory, PathBuf::from(DIRECTORY)));
        let sessions = port.sessions()?;
        let session = sessions.first().ok_or("no session")?;
        let read = port.read_from(session, Cursor::default())?;
        let cursor = read.cursor();
        let stream: Stream = read.into_iter().collect();
        Ok(Recorded { records: SessionRecords::read(session.stem(), cursor, &stream), cursor })
    }
}

#[test]
fn a_written_session_reads_back_with_its_segments_links_and_cursor() -> Result<(), Box<dyn Error>> {
    let Recorded { records, cursor } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let shown = store.read(ShownMessage::named(Uuid::new("o1".to_owned())))?.ok_or("o1 was not stored")?;
    let message = shown.message();
    assert_eq!(
        [
            message.uuid().to_string(),
            message.author().to_string(),
            message.session().to_string(),
            message.stamp().to_string()
        ],
        ["o1", "owner", "s", "2026-01-01T00:00:00Z"]
    );
    let segments: Vec<String> =
        shown.segments().iter().map(|segment| format!("{} {}", segment.ordinal(), segment.shape())).collect();
    assert_eq!(segments, ["0 item", "1 item"]);
    let first = shown.segments().first().map(Body::from).ok_or("no segment")?;
    assert_eq!(first.to_string(), "1. go");
    let links: Vec<String> = shown
        .links()
        .iter()
        .map(|link| {
            let [source, target] = link.anchors();
            format!("{source} {} {target}", link.relation())
        })
        .collect();
    assert_eq!(links, ["l1 replies-to o1"]);
    let cursors = store.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), cursor);
    Ok(())
}

#[test]
fn writing_the_same_session_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let first = store.write(Recorded::session()?.records)?;
    let second = store.write(Recorded::session()?.records)?;
    assert!(first.count() > 0);
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn an_unknown_message_is_not_shown() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(Recorded::session()?.records)?;
    assert!(store.read(ShownMessage::named(Uuid::new("nobody".to_owned())))?.is_none());
    Ok(())
}

struct Said;

impl Said {
    fn owner(uuid: &str, second: usize, text: &str) -> String {
        format!(
            "{{\"type\":\"user\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:0{second}Z\",\"origin\":{{\"kind\":\"human\"}},\"message\":{{\"content\":\"{text}\"}}}}\n"
        )
    }

    fn lead(uuid: &str, text: &str) -> String {
        format!(
            "{{\"type\":\"assistant\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:09Z\",\"origin\":null,\"message\":{{\"model\":\"claude\",\"content\":[{{\"type\":\"text\",\"text\":\"{text}\"}}]}}}}\n"
        )
    }

    fn found(lines: &[String], words: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        Said::scoped(lines, words, Scope::Owner)
    }

    fn scoped(lines: &[String], words: &[&str], scope: Scope) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(Recorded::of(&lines.concat())?.records)?;
        let terms = Terms::lexed(words.iter().copied()).ok_or("no terms")?;
        let hits = store.read(SearchSegments::matching(terms, scope))?;
        Ok(hits
            .iter()
            .map(|hit| format!("{} {} {} {}", hit.uuid(), hit.author(), hit.stamp(), Body::from(hit)))
            .collect())
    }
}

#[test]
fn a_search_returns_each_segment_holding_the_word_with_its_message() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "alpha here"), Said::owner("o2", 2, "beta there")];
    assert_eq!(Said::found(&lines, &["beta"])?, ["o2 owner 2026-01-01T00:00:02Z beta there"]);
    Ok(())
}

#[test]
fn several_terms_rank_a_segment_dense_in_the_rarer_term_first() -> Result<(), Box<dyn Error>> {
    let lines = [
        Said::owner("o1", 1, "alpha alpha alpha alpha beta"),
        Said::owner("o2", 2, "alpha beta beta beta beta"),
        Said::owner("o3", 3, "alpha"),
        Said::owner("o4", 4, "gamma"),
        Said::owner("o5", 5, "gamma"),
        Said::owner("o6", 6, "gamma"),
        Said::owner("o7", 7, "gamma"),
        Said::owner("o8", 8, "gamma"),
    ];
    let found = Said::found(&lines, &["alpha", "beta"])?;
    assert_eq!(
        found,
        [
            "o2 owner 2026-01-01T00:00:02Z alpha beta beta beta beta",
            "o1 owner 2026-01-01T00:00:01Z alpha alpha alpha alpha beta"
        ]
    );
    Ok(())
}

#[test]
fn several_terms_rank_a_segment_dense_in_both_before_one_diluted() -> Result<(), Box<dyn Error>> {
    let lines = [
        Said::owner("o1", 1, "delta epsilon zeta zeta zeta zeta zeta zeta zeta zeta"),
        Said::owner("o2", 2, "delta epsilon"),
        Said::owner("o3", 3, "gamma"),
        Said::owner("o4", 4, "gamma"),
        Said::owner("o5", 5, "gamma"),
        Said::owner("o6", 6, "gamma"),
        Said::owner("o7", 7, "gamma"),
        Said::owner("o8", 8, "gamma"),
    ];
    let found = Said::found(&lines, &["delta", "epsilon"])?;
    assert_eq!(found.first().map(String::as_str), Some("o2 owner 2026-01-01T00:00:02Z delta epsilon"));
    Ok(())
}

#[test]
fn the_lead_is_found_only_in_the_everyone_scope() -> Result<(), Box<dyn Error>> {
    let lines = [Said::owner("o1", 1, "omega"), Said::lead("l1", "omega")];
    assert_eq!(Said::scoped(&lines, &["omega"], Scope::Owner)?, ["o1 owner 2026-01-01T00:00:01Z omega"]);
    assert_eq!(Said::scoped(&lines, &["omega"], Scope::Everyone)?.len(), 2);
    Ok(())
}

#[derive(Clone, Debug)]
struct Dag {
    facets: Vec<Facet>,
    edges: Vec<Edge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    child: usize,
    parent: usize,
}

impl Dag {
    fn any() -> impl Strategy<Value = Dag> {
        vec((vec(any::<Index>(), 0..4), select(Facet::VARIANTS)), 1..16).prop_flat_map(|choices| {
            let facets: Vec<Facet> = choices.iter().map(|(_parents, facet)| *facet).collect();
            let edges: BTreeSet<Edge> = choices
                .iter()
                .enumerate()
                .skip(1)
                .flat_map(|(child, (parents, _facet))| {
                    parents.iter().map(move |index| Edge { child, parent: index.index(child) })
                })
                .collect();
            Just(edges.into_iter().collect::<Vec<Edge>>())
                .prop_shuffle()
                .prop_map(move |edges| Dag { facets: facets.clone(), edges })
        })
    }

    fn id(node: usize) -> NodeId {
        NodeId::new(i64::try_from(node).unwrap_or(i64::MAX).saturating_add(1))
    }

    fn node(facet: Facet) -> Result<NewNode, Box<dyn Error>> {
        Ok(NewNode::from(Entry::new("p".parse()?, facet, Stage::Vague, Provenance::from(Entered::Init))))
    }

    fn planted(&self, store: &Port<'_, Sqlite>) -> Result<Written, Box<dyn Error>> {
        let nodes = self
            .facets
            .iter()
            .map(|facet| Ok(store.write(Dag::node(*facet)?)?))
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        let links = self
            .edges
            .iter()
            .map(|edge| {
                Ok(store.write(Linked::new(
                    Dag::id(edge.child),
                    NodeRelation::Refines,
                    Dag::id(edge.parent),
                    Entered::Init.into(),
                ))?)
            })
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        Ok(Written::total(nodes.into_iter().chain(links)))
    }

    fn parents(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.edges.iter().filter(move |edge| edge.child == node).map(|edge| edge.parent)
    }

    fn ancestors(&self, node: usize) -> Vec<NodeId> {
        let levels = std::iter::successors(Some(BTreeSet::from([node])), |frontier| {
            Some(frontier.iter().flat_map(|below| self.parents(*below)).collect::<BTreeSet<usize>>())
        })
        .take(self.facets.len());
        let depths = levels
            .enumerate()
            .skip(1)
            .flat_map(|(depth, level)| level.into_iter().map(move |above| (above, depth)))
            .fold(BTreeMap::new(), |mut depths, (above, depth)| {
                depths.entry(above).or_insert(depth);
                depths
            });
        let ordered: BTreeSet<(Reverse<usize>, usize)> =
            depths.into_iter().map(|(above, depth)| (Reverse(depth), above)).collect();
        ordered.into_iter().map(|(_depth, above)| Dag::id(above)).collect()
    }

    fn children(&self, node: usize) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|edge| edge.parent == node)
            .map(|edge| edge.child)
            .collect::<BTreeSet<usize>>()
            .into_iter()
            .map(Dag::id)
            .collect()
    }
}

struct Ids;

impl Ids {
    fn kin(placed: &Placed, kin: Kin) -> Vec<NodeId> {
        placed
            .relatives()
            .iter()
            .filter(|relative| relative.kin() == kin)
            .map(|relative| relative.node().identity())
            .collect()
    }

    fn earlier(seen: &Seen) -> Option<i64> {
        match seen {
            Seen::Before(earlier) => Some(earlier.identity().into_inner()),
            Seen::Unseen(_) => None,
        }
    }
}

laws::law!(the_closure_holds_exactly_the_transitive_refinements, Dag::any(), dag, {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    dag.planted(&store).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    (0..dag.facets.len()).try_for_each(|node| {
        let placed = store
            .read(ShownNode::from(Dag::id(node)))
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?
            .ok_or_else(|| TestCaseError::fail("a planted node is not shown"))?;
        prop_assert_eq!(Ids::kin(&placed, Kin::Ancestor), dag.ancestors(node));
        prop_assert_eq!(Ids::kin(&placed, Kin::Child), dag.children(node));
        prop_assert_eq!(placed.node().entry().facet(), dag.facets.get(node).copied().unwrap_or(Facet::Project));
        Ok(())
    })?;
});

struct Projects;

impl Projects {
    fn rooted(store: &Port<'_, Sqlite>, name: &str) -> Result<NodeId, Box<dyn Error>> {
        let project: Project = name.parse()?;
        store.write(NewNode::from(Entry::new(project.clone(), Facet::Project, Stage::Vague, Entered::Init.into())))?;
        Ok(store.read(ProjectRoot::from(project))?.ok_or("no root")?)
    }
}

laws::law!(
    a_project_root_is_found_by_its_project_alone,
    select(Facet::VARIANTS).prop_filter("a facet other than a project root", |facet| *facet != Facet::Project),
    facet,
    {
        let found = Projects::alone(facet).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        prop_assert_eq!(found, [Some(NodeId::new(3)), None]);
    }
);

impl Projects {
    fn alone(facet: Facet) -> Result<[Option<NodeId>; 2], Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        Projects::rooted(&store, "q")?;
        store.write(Dag::node(facet)?)?;
        store.write(NewNode::from(Entry::new("p".parse()?, Facet::Project, Stage::Vague, Entered::Init.into())))?;
        Ok([
            store.read(ProjectRoot::from("p".parse::<Project>()?))?,
            store.read(ProjectRoot::from("r".parse::<Project>()?))?,
        ])
    }
}

#[test]
fn a_consumes_link_is_listed_from_its_source_and_adds_no_ancestor() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let eater = Projects::rooted(&store, "p")?;
    let consumed = Projects::rooted(&store, "q")?;
    store.write(Linked::new(eater, NodeRelation::Consumes, consumed, Entered::Init.into()))?;
    let shown = store.read(ShownNode::from(eater))?.ok_or("no consumer shown")?;
    let links: Vec<(NodeRelation, NodeId)> =
        shown.links().iter().map(|link| (link.relation(), link.target())).collect();
    assert_eq!(links, [(NodeRelation::Consumes, consumed)]);
    assert_eq!(Ids::kin(&shown, Kin::Ancestor), []);
    assert_eq!(Ids::kin(&store.read(ShownNode::from(consumed))?.ok_or("no consumed shown")?, Kin::Child), []);
    Ok(())
}

#[test]
fn linking_the_same_nodes_again_writes_no_row() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let parent = Projects::rooted(&store, "p")?;
    store.write(Dag::node(Facet::Project)?)?;
    let child = NodeId::new(parent.into_inner().saturating_add(1));
    let first = store.write(Linked::new(child, NodeRelation::Refines, parent, Entered::Init.into()))?;
    let second = store.write(Linked::new(child, NodeRelation::Refines, parent, Entered::Init.into()))?;
    assert!(first > Written::zero());
    assert_eq!(second, Written::zero());
    Ok(())
}

#[test]
fn a_shortcut_refinement_is_listed_as_a_child() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let edges = [(1, 0), (2, 1), (2, 0)].map(|(child, parent)| Edge { child, parent });
    Dag { facets: vec![Facet::Project; 3], edges: edges.to_vec() }.planted(&store)?;
    let shown = store.read(ShownNode::from(Dag::id(0)))?.ok_or("no root shown")?;
    assert_eq!(Ids::kin(&shown, Kin::Child), [Dag::id(1), Dag::id(2)]);
    Ok(())
}

#[test]
fn an_unknown_node_is_not_shown() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    Projects::rooted(&store, "p")?;
    assert!(store.read(ShownNode::from(NodeId::new(99)))?.is_none());
    Ok(())
}

struct Proposals;

impl Proposals {
    fn node(id: i64) -> Result<Node, Box<dyn Error>> {
        let entry = Entry::new("p".parse()?, Facet::Feature, Stage::Vague, Entered::Apply.into());
        Ok(Node::new(NodeId::new(id), entry).titled(Title::new(format!("child {id}"))))
    }

    fn child(parent: NodeId, id: i64) -> Result<Directed, Box<dyn Error>> {
        Ok(Directed::Proposed { node: Proposals::node(id)?, parent })
    }

    fn superseded(old: i64, by: i64) -> Result<Directed, Box<dyn Error>> {
        Ok(Directed::Superseded(Supersession::within(&Proposals::node(old)?, &Proposals::node(by)?)?))
    }

    fn written(store: &Port<'_, Sqlite>, directed: Vec<Directed>) -> Result<Written, Box<dyn Error>> {
        let file = Proposals::unseen(store, &format!("{directed:?}"))?;
        Ok(store.write(Directions::new(Provenance::from(Entered::Apply), file, directed))?)
    }

    fn unseen(store: &Port<'_, Sqlite>, contents: &str) -> Result<Unseen, Box<dyn Error>> {
        match store.read(AppliedFile::from(Bytes::from(contents)))? {
            Seen::Unseen(unseen) => Ok(unseen),
            Seen::Before(earlier) => {
                Err(format!("{contents} was applied as file {}", earlier.identity().into_inner()).into())
            }
        }
    }

    fn segment(message: &str, ordinal: usize) -> Segment {
        Segment::new(Uuid::new(message.to_owned()), Ordinal::new(ordinal))
    }
}

#[test]
fn a_proposal_plants_a_titled_vague_child_under_its_parent() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let next = store.read(NextNode)?;
    Proposals::written(&store, vec![Proposals::child(root, next.into_inner())?])?;
    let child = store.read(ShownNode::from(next))?.ok_or("the proposal was not planted")?;
    assert_eq!(child.node().entry().stage(), Stage::Vague);
    assert_eq!(child.node().title(), Some(&Title::new("child 2".to_owned())));
    assert_eq!(Ids::kin(&child, Kin::Ancestor), [root]);
    assert_eq!(Ids::kin(&store.read(ShownNode::from(root))?.ok_or("no root")?, Kin::Child), [next]);
    Ok(())
}

#[test]
fn the_next_node_follows_the_highest_node() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    assert_eq!(store.read(NextNode)?, NodeId::new(1));
    Projects::rooted(&store, "p")?;
    Projects::rooted(&store, "q")?;
    assert_eq!(store.read(NextNode)?, NodeId::new(3));
    Ok(())
}

#[test]
fn directions_are_written_whole_or_not_at_all() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let root = Projects::rooted(&store, "p")?;
    let stray = Directed::Conversational { segment: Proposals::segment("nowhere", 0) };
    assert!(Proposals::written(&store, vec![Proposals::child(root, 2)?, stray]).is_err());
    assert!(store.read(ShownNode::from(NodeId::new(2)))?.is_none());
    Ok(())
}

#[test]
fn a_file_is_seen_once_its_directions_are_written_and_not_when_they_are_refused() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let root = Projects::rooted(&store, "p")?;
    let stray = Directed::Conversational { segment: Proposals::segment("nowhere", 0) };
    let refused = Proposals::unseen(&store, "refused")?;
    assert!(store.write(Directions::new(Provenance::from(Entered::Apply), refused, vec![stray])).is_err());
    assert!(matches!(store.read(AppliedFile::from(Bytes::from("refused")))?, Seen::Unseen(_)));
    let kept = Proposals::unseen(&store, "kept")?;
    store.write(Directions::new(Provenance::from(Entered::Apply), kept, vec![Proposals::child(root, 2)?]))?;
    let other = Proposals::unseen(&store, "other")?;
    store.write(Directions::new(Provenance::from(Entered::Apply), other, Vec::new()))?;
    let seen = [Bytes::from("kept"), Bytes::from("other"), Bytes::from("kept ")]
        .map(|contents| store.read(AppliedFile::from(contents)).map(|seen| Ids::earlier(&seen)))
        .into_iter()
        .collect::<Result<Vec<Option<i64>>, _>>()?;
    assert_eq!(seen, [Some(1), Some(2), None]);
    Ok(())
}

#[test]
fn a_superseded_node_stays_shown_and_its_successor_links_to_it() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    Proposals::written(&store, vec![Proposals::child(root, 2)?, Proposals::child(root, 3)?])?;
    Proposals::written(&store, vec![Proposals::superseded(2, 3)?])?;
    let successor = store.read(ShownNode::from(NodeId::new(3)))?.ok_or("no successor")?;
    let links: Vec<(NodeRelation, NodeId, Provenance)> =
        successor.links().iter().map(|link| (link.relation(), link.target(), link.provenance())).collect();
    let applied = Provenance::from(Entered::Apply);
    assert_eq!(links, [(NodeRelation::Refines, root, applied), (NodeRelation::Supersedes, NodeId::new(2), applied)]);
    assert!(store.read(ShownNode::from(NodeId::new(2)))?.is_some());
    Ok(())
}

#[test]
fn a_prefix_finds_every_segment_of_the_messages_it_starts() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let heard = |prefix: &str| -> Result<Vec<(Segment, Authorship)>, Box<dyn Error>> {
        let spoken = store.read(Prefixed::from(Prefix::new(prefix.to_owned())))?;
        Ok(spoken.iter().map(|spoken| (spoken.segment().clone(), spoken.author())).collect())
    };
    let owner = [(Proposals::segment("o1", 0), Authorship::Owner), (Proposals::segment("o1", 1), Authorship::Owner)];
    assert_eq!(heard("o")?, owner);
    assert_eq!(heard("o1")?, owner);
    assert_eq!(heard("l1")?, [(Proposals::segment("l1", 0), Authorship::Lead)]);
    assert_eq!(heard("1")?, []);
    Ok(())
}

#[test]
fn a_message_is_covered_where_any_annotation_names_its_segments() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let [asked, answered, said] =
        [("o1", 0), ("o1", 1), ("l1", 0)].map(|(message, ordinal)| Proposals::segment(message, ordinal));
    let covered = |messages: &[&str]| -> Result<Vec<Segment>, Box<dyn Error>> {
        let messages = messages.iter().map(|message| Uuid::new((*message).to_owned()));
        Ok(store.read(messages.collect::<Covered>())?.into_iter().collect())
    };
    assert_eq!(covered(&["o1", "l1"])?, []);
    let related =
        Directed::Related { source: said.clone(), target: answered.clone(), relation: SegmentRelation::Answers };
    Proposals::written(&store, vec![Directed::Conversational { segment: asked.clone() }, related])?;
    assert_eq!(covered(&["o1"])?, [asked.clone(), answered.clone()]);
    assert_eq!(covered(&["l1"])?, std::slice::from_ref(&said));
    assert_eq!(covered(&["o1", "l1"])?, [said, asked, answered]);
    Ok(())
}

const SEGMENTED: &str = concat!(
    "{\"type\":\"user\",\"uuid\":\"o1\",\"parentUuid\":null,\"sessionId\":\"s\",",
    "\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{\"kind\":\"human\"},",
    "\"message\":{\"content\":\"1. a\\n2. b\\n3. c\\n4. d\\n5. e\"}}\n"
);

const ENDS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closed {
    Refines,
    Supersedes,
    SegmentSupersedes,
    SegmentCorrects,
}

#[derive(Clone, Copy, Debug)]
enum Ends {
    Nodes(NodeRelation),
    Segments(SegmentRelation),
}

#[derive(Clone, Debug)]
struct Probe {
    closed: Closed,
    edges: Vec<Edge>,
    tried: Edge,
}

impl Closed {
    const ALL: [Closed; 4] = [Closed::Refines, Closed::Supersedes, Closed::SegmentSupersedes, Closed::SegmentCorrects];

    fn ends(self) -> Ends {
        match self {
            Closed::Refines => Ends::Nodes(NodeRelation::Refines),
            Closed::Supersedes => Ends::Nodes(NodeRelation::Supersedes),
            Closed::SegmentSupersedes => Ends::Segments(SegmentRelation::Supersedes),
            Closed::SegmentCorrects => Ends::Segments(SegmentRelation::Corrects),
        }
    }
}

impl Ends {
    fn linked(self, store: &Port<'_, Sqlite>, edge: Edge, file: Unseen) -> Result<Written, Refusal> {
        match self {
            Ends::Nodes(relation) => {
                store.write(Linked::new(Dag::id(edge.child), relation, Dag::id(edge.parent), Entered::Init.into()))
            }
            Ends::Segments(relation) => store.write(Directions::new(
                Provenance::from(Entered::Apply),
                file,
                vec![Directed::Related {
                    source: Proposals::segment("o1", edge.child),
                    target: Proposals::segment("o1", edge.parent),
                    relation,
                }],
            )),
        }
    }

    fn cycle(self, edge: Edge) -> Cycle {
        match self {
            Ends::Nodes(relation) => {
                Cycle::Nodes { source: Dag::id(edge.child), relation, target: Dag::id(edge.parent) }
            }
            Ends::Segments(relation) => Cycle::Segments {
                source: Proposals::segment("o1", edge.child),
                relation,
                target: Proposals::segment("o1", edge.parent),
            },
        }
    }
}

impl Probe {
    fn any() -> impl Strategy<Value = Probe> {
        let edges = vec((0..ENDS, 0..ENDS), 0..10).prop_map(|pairs| {
            pairs
                .into_iter()
                .filter(|(first, second)| first != second)
                .map(|(first, second)| Edge { child: first.max(second), parent: first.min(second) })
                .collect()
        });
        (select(Closed::ALL.to_vec()), edges, 0..ENDS, 0..ENDS).prop_map(|(closed, edges, child, parent)| Probe {
            closed,
            edges,
            tried: Edge { child, parent },
        })
    }

    fn refused(&self) -> Result<Option<Cycle>, Box<dyn Error>> {
        let Recorded { records, .. } = Recorded::of(SEGMENTED)?;
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        store.write(records)?;
        (0..ENDS)
            .map(|_node| Ok(store.write(Dag::node(Facet::Feature)?)?))
            .collect::<Result<Vec<Written>, Box<dyn Error>>>()?;
        let ends = self.closed.ends();
        let mut files = (0..=self.edges.len())
            .map(|nth| Proposals::unseen(&store, &nth.to_string()))
            .collect::<Result<Vec<Unseen>, Box<dyn Error>>>()?
            .into_iter();
        let tried = files.next().ok_or("no file for the tried edge")?;
        self.edges
            .iter()
            .zip(files)
            .map(|(edge, file)| ends.linked(&store, *edge, file))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Probe::cyclic(ends.linked(&store, self.tried, tried))
    }

    fn cyclic(written: Result<Written, Refusal>) -> Result<Option<Cycle>, Box<dyn Error>> {
        match written {
            Ok(_written) => Ok(None),
            Err(Refusal::Cyclic(cycle)) => Ok(Some(cycle)),
            Err(failure) => Err(failure.into()),
        }
    }

    fn reached(&self) -> BTreeSet<usize> {
        std::iter::successors(Some(BTreeSet::from([self.tried.parent])), |frontier| {
            let next: BTreeSet<usize> =
                self.edges.iter().filter(|edge| frontier.contains(&edge.child)).map(|edge| edge.parent).collect();
            Some(frontier.union(&next).copied().collect())
        })
        .nth(ENDS)
        .unwrap_or_default()
    }

    fn chain() -> Vec<Edge> {
        vec![Edge { child: 1, parent: 0 }, Edge { child: 2, parent: 1 }]
    }

    fn closing() -> Edge {
        Edge { child: 0, parent: 2 }
    }

    fn wanted(&self) -> Option<Cycle> {
        self.reached().get(&self.tried.child).map(|_reached| self.closed.ends().cycle(self.tried))
    }
}

laws::law!(an_edge_is_refused_exactly_where_the_closure_holds_its_reverse_path, Probe::any(), probe, {
    let refused = probe.refused().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(refused, probe.wanted());
});

#[test]
fn an_edge_reversing_a_written_path_is_refused_for_every_acyclic_relation() {
    let refused = Closed::ALL
        .map(|closed| Probe { closed, edges: Probe::chain(), tried: Probe::closing() }.refused().ok().flatten());
    assert_eq!(refused, Closed::ALL.map(|closed| Some(closed.ends().cycle(Probe::closing()))));
}

struct Runs;

impl Runs {
    fn spawned(pid: u32, deadline: i64) -> Spawned {
        let started = "Fri Nov  6 01:16:00 2026".parse::<Started>().ok();
        Spawned::new(Pid::new(pid), started, Deadline::new(Wall::new(deadline)))
    }

    fn usage(counts: [u64; 4]) -> Usage {
        let [input, output, cache_written, cache_read] = counts.map(Tokens::new);
        Usage::new(input, output, cache_written, cache_read)
    }
}

#[test]
fn an_owner_segment_is_unstructured_until_an_annotation_names_it() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    let [asked, answered, said] =
        [("o1", 0), ("o1", 1), ("l1", 0)].map(|(message, ordinal)| Proposals::segment(message, ordinal));
    let unstructured = || -> Result<Vec<Segment>, Box<dyn Error>> {
        let unplaced = store.read(Unstructured)?;
        Ok(unplaced.iter().map(|unplaced| AsRef::<Segment>::as_ref(unplaced).clone()).collect())
    };
    let first = store.read(Unstructured)?.first().map(|unplaced| AsRef::<Body>::as_ref(unplaced).to_string());
    assert_eq!(first.as_deref(), Some("1. go"));
    assert_eq!(unstructured()?, [asked.clone(), answered.clone()]);
    Proposals::written(&store, vec![Directed::Conversational { segment: asked }])?;
    assert_eq!(unstructured()?, std::slice::from_ref(&answered));
    Proposals::written(
        &store,
        vec![Directed::Related { source: said, target: answered, relation: SegmentRelation::Answers }],
    )?;
    assert_eq!(unstructured()?, []);
    Ok(())
}

#[test]
fn a_spawned_run_is_live_until_it_ends_and_its_job_is_charged_its_usage() -> Result<(), Box<dyn Error>> {
    let Recorded { records, .. } = Recorded::session()?;
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    store.write(records)?;
    assert_eq!((store.read(NextJob)?, store.read(NextRun)?), (JobId::new(1), RunId::new(1)));
    let job = JobId::new(1);
    let opened = store.write(JobOpened::new(job, vec![Proposals::segment("o1", 0), Proposals::segment("o1", 1)]))?;
    assert_eq!(opened, Written::new(3));
    store.write(RunSpawned::new(RunId::new(1), job, Runs::spawned(42, 100)))?;
    store.write(RunSpawned::new(RunId::new(2), job, Runs::spawned(43, 200)))?;
    let live: Vec<(RunId, Spawned)> =
        store.read(LiveRuns)?.iter().map(|stored| (stored.identity(), stored.spawned().clone())).collect();
    assert_eq!(live, [(RunId::new(1), Runs::spawned(42, 100)), (RunId::new(2), Runs::spawned(43, 200))]);
    assert_eq!((store.read(NextJob)?, store.read(NextRun)?), (JobId::new(2), RunId::new(3)));
    let ended = [
        store.write(RunEnded::new(RunId::new(1), RunState::Finished, Some(Runs::usage([11, 7, 3, 5]))))?,
        store.write(RunEnded::new(RunId::new(2), RunState::Overdue, None))?,
        store.write(JobClosed::new(job, JobState::Applied))?,
    ];
    assert_eq!(ended, [Written::new(1); 3]);
    assert_eq!(store.read(LiveRuns)?, []);
    assert_eq!(store.read(SpentOn::from(job))?, Runs::usage([11, 7, 3, 5]));
    assert_eq!(store.read(SpentOn::from(JobId::new(9)))?, Usage::default());
    Ok(())
}

#[test]
fn context_lists_the_project_roots_first_then_the_newest_nodes() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    Projects::rooted(&store, "q")?;
    Proposals::written(&store, vec![Proposals::child(root, 3)?, Proposals::child(root, 4)?])?;
    let listed: Vec<NodeId> = store.read(ContextNodes)?.iter().map(Node::identity).collect();
    assert_eq!(listed, [2, 1, 4, 3].map(NodeId::new));
    Ok(())
}

#[test]
fn an_agent_provenance_is_kept_on_what_it_wrote() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let root = Projects::rooted(&store, "p")?;
    let agent = Provenance::Agent(RunId::new(7));
    let unseen = Proposals::unseen(&store, "agent")?;
    store.write(Directions::new(agent, unseen, vec![Proposals::child(root, 2)?]))?;
    let child = store.read(ShownNode::from(NodeId::new(2)))?.ok_or("no child")?;
    let links: Vec<Provenance> = child.links().iter().map(|link| link.provenance()).collect();
    assert_eq!(links, [agent]);
    Ok(())
}
