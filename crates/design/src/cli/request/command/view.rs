use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::iter;

use design_model::{NodeId, Outline, Segment, SpanRelation};
use design_store::{
    ContextNodes, CurrentSpans, Ended, ImportedEnding, ImportedNotes, Noted, Placement, ProjectRoot, Resolved,
    UnplacedSegments,
};
use file_ports::Files;
use grammar::notes::{self, Notes};
use named_derive::Named;
use newtype::Wrapping;
use store_ports::Store;

use super::printed::Printed;
use super::rendering::Refs;
use super::subject::Found;
use crate::Outcome;
use crate::cli::Viewed;
use crate::cli::failure::Failure;
use crate::cli::output::Line;
use crate::cli::request::{Ground, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Tree(Tree, Root),
    Unplaced,
    Notes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tree {
    CurrentDecision,
    OpenQuestions,
    Outline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Root {
    Configured,
    Node(NodeId),
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Lens {
    #[name("current-decision")]
    CurrentDecision,
    #[name("open-questions")]
    OpenQuestions,
    #[name("outline")]
    Outline,
    #[name("unplaced")]
    Unplaced,
    #[name("notes")]
    Notes,
}

#[derive(Debug)]
struct Subtree(Vec<Outline>);

impl Lens {
    fn spoken(token: &Token) -> Result<Lens, Failure> {
        token.text().and_then(|word| word.parse().ok()).ok_or(Failure::Usage)
    }

    fn tree(self) -> Result<Tree, View> {
        match self {
            Lens::CurrentDecision => Ok(Tree::CurrentDecision),
            Lens::OpenQuestions => Ok(Tree::OpenQuestions),
            Lens::Outline => Ok(Tree::Outline),
            Lens::Unplaced => Err(View::Unplaced),
            Lens::Notes => Err(View::Notes),
        }
    }

    fn configured(self) -> View {
        self.tree().map_or_else(|whole| whole, |tree| View::Tree(tree, Root::Configured))
    }

    fn rooted(self, given: &Token) -> Result<View, Failure> {
        let tree = self.tree().map_err(|_whole| Failure::Usage)?;
        let node = given.text().ok_or(Failure::Usage)?.parse()?;
        Ok(View::Tree(tree, Root::Node(node)))
    }
}

impl View {
    const ROOTED: usize = 2;

    pub fn read(rest: &[Token]) -> Result<View, Failure> {
        <&[Token; 1]>::try_from(rest)
            .map_err(|_count| Failure::Usage)
            .and_then(|[lens]| Lens::spoken(lens).map(Lens::configured))
            .or_else(|_unworded| {
                <&[Token; View::ROOTED]>::try_from(rest)
                    .map_err(|_count| Failure::Usage)
                    .and_then(|[lens, node]| Lens::spoken(lens)?.rooted(node))
            })
    }

    pub fn written<F: Files, S: Store>(
        self,
        ground: &Ground<'_, F, S>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        self.printed(ground, out).map(|_printed| Outcome::Viewed)
    }

    fn printed<F: Files, S: Store>(self, ground: &Ground<'_, F, S>, out: &mut impl Write) -> Result<Printed, Failure> {
        match self {
            View::Unplaced => View::unplaced(ground.store()).and_then(|lines| Printed::lines(out, &lines)),
            View::Tree(tree, root) => root
                .subtree(ground)
                .and_then(|subtree| tree.lines(ground.store(), &subtree))
                .and_then(|lines| Printed::lines(out, &lines)),
            View::Notes => View::notes(ground).and_then(|notes| Printed::rendered(out, &notes)),
        }
    }

    fn notes<F: Files, S: Store>(ground: &Ground<'_, F, S>) -> Result<Notes, Failure> {
        let project = ground.located().project().ok_or(Failure::Unconfigured)?;
        let noted: Vec<Noted> = ground.store().read(ImportedNotes::from(project.clone()))?;
        let ending = ground.store().read(ImportedEnding::from(project.clone()))?;
        Ok(Notes::gathered(
            noted.iter().map(|note| notes::Line::read(note.line().into_inner(), &note.title().clone().into_inner())),
            ending.unwrap_or_default(),
        ))
    }

    fn unplaced(store: &impl Store) -> Result<Vec<Line>, Failure> {
        let unplaced = store.read(UnplacedSegments)?;
        let segments = unplaced.iter().map(|segment| AsRef::<Segment>::as_ref(segment).clone());
        Ok(Refs::read(store, segments)?.unplaced(&unplaced))
    }
}

impl Root {
    fn subtree<F: Files, S: Store>(self, ground: &Ground<'_, F, S>) -> Result<Subtree, Failure> {
        let root = self.resolved(ground)?;
        Subtree::read(ground.store(), root)
    }

    fn resolved<F: Files, S: Store>(self, ground: &Ground<'_, F, S>) -> Result<Resolved, Failure> {
        match self {
            Root::Node(node) => Found::resolved(ground.store(), node),
            Root::Configured => Root::configured(ground),
        }
    }

    fn configured<F: Files, S: Store>(ground: &Ground<'_, F, S>) -> Result<Resolved, Failure> {
        let project = ground.located().project().ok_or(Failure::Unconfigured)?;
        let root = ground.store().read(ProjectRoot::from(project.clone()))?;
        root.ok_or_else(|| Failure::Unrooted(project.clone()))
    }
}

impl Tree {
    fn lines(self, store: &impl Store, subtree: &Subtree) -> Result<Vec<Line>, Failure> {
        match self {
            Tree::Outline => Ok(subtree.outlined()),
            Tree::CurrentDecision => subtree.standing(store, SpanRelation::Decides),
            Tree::OpenQuestions => subtree.standing(store, SpanRelation::Questions),
        }
    }
}

impl Subtree {
    fn read(store: &impl Store, root: Resolved) -> Result<Subtree, Failure> {
        let node = root.identity();
        store.read(Ended::from(root))?.map(|ending| Failure::Ended { node, ending }).map_or(Ok(()), Err)?;
        let live = store.read(ContextNodes)?;
        let children = live.iter().fold(BTreeMap::<NodeId, Vec<&Outline>>::new(), |mut children, outline| {
            outline.parent().into_iter().for_each(|parent| children.entry(parent).or_default().push(outline));
            children
        });
        let mut pending: Vec<&Outline> = live.iter().filter(|outline| outline.node() == node).collect();
        let ordered = iter::from_fn(|| {
            let next = pending.pop()?;
            pending.extend(children.get(&next.node()).into_iter().flatten().rev());
            Some(next.clone())
        });
        Ok(Subtree(ordered.collect()))
    }

    fn outlined(&self) -> Vec<Line> {
        self.0.iter().map(|outline| Line::Viewed(Viewed::Outlined(outline.clone()))).collect()
    }

    fn standing(&self, store: &impl Store, relation: SpanRelation) -> Result<Vec<Line>, Failure> {
        let nodes: BTreeSet<NodeId> = self.0.iter().map(Outline::node).collect();
        let current = store.read(CurrentSpans::from(relation))?;
        let spans: Vec<Placement> = current.into_iter().filter(|span| nodes.contains(&span.node())).collect();
        Ok(Refs::read(store, spans.iter().map(Placement::segment))?.standing(&spans))
    }
}
