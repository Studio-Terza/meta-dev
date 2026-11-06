use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use design_model::{Entered, Entry, Facet, Kin, NodeId, NodeRelation, Project, Provenance, Stage};
use design_store::{
    Linked, NewNode, Placed, ProjectRoot, Scope, SearchSegments, SessionCursors, SessionRecords, ShownMessage,
    ShownNode, StreamSchema,
};
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
use transcript_ports::{Body, ClaudeCodeDir, Cursor, SessionId, Transcripts, Uuid};

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
                Ok(store.write(Linked::new(Dag::id(edge.child), NodeRelation::Refines, Dag::id(edge.parent)))?)
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
    store.write(Linked::new(eater, NodeRelation::Consumes, consumed))?;
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
    let first = store.write(Linked::new(child, NodeRelation::Refines, parent))?;
    let second = store.write(Linked::new(child, NodeRelation::Refines, parent))?;
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
