use algebra::Semigroup;
use lint_layout::Name;
use std::path::PathBuf;

use lint_placement::{CrateRoot, Krate, Level, Nest};

struct Tree {
    krate: Name,
    other: Name,
    segments: Vec<Name>,
}

impl Tree {
    fn new() -> Tree {
        Tree {
            krate: Name::from("graph"),
            other: Name::from("lint"),
            segments: vec![Name::from("model"), Name::from("building")],
        }
    }

    fn at(&self, depth: usize) -> Nest<'_> {
        Nest::module(Krate::Named(&self.krate), self.segments.get(..depth).unwrap_or(&self.segments))
    }
}

#[test]
fn a_module_climbs_through_its_crate_to_the_workspace() {
    let tree = Tree::new();
    let lineage: Vec<String> = tree.at(2).lineage().map(|nest| nest.to_string()).collect();
    assert_eq!(lineage, ["graph::model::building", "graph::model", "crate graph", "the workspace"]);
    assert_eq!(Nest::WORKSPACE.parent(), None);
}

#[test]
fn each_place_knows_its_level() {
    let tree = Tree::new();
    assert_eq!(
        [tree.at(2).level(), tree.at(0).level(), Nest::WORKSPACE.level()],
        [Level::Module, Level::Crate, Level::Workspace]
    );
}

#[test]
fn an_outer_place_holds_its_inner_ones_and_not_its_neighbours() {
    let tree = Tree::new();
    let other = Nest::module(Krate::Named(&tree.other), &tree.segments);
    assert_eq!(tree.at(2).lineage().find(|outer| *outer == tree.at(1)), Some(tree.at(1)));
    assert_eq!(tree.at(1).lineage().find(|outer| *outer == tree.at(2)), None);
    assert_eq!(other.lineage().find(|outer| *outer == tree.at(1)), None);
    assert_eq!(tree.at(2).combine(other), Nest::WORKSPACE);
    assert_eq!(tree.at(2).combine(tree.at(1)), tree.at(1));
}

#[test]
fn a_place_is_named_by_its_last_segment_or_its_crate() {
    let tree = Tree::new();
    assert_eq!(tree.at(2).named(&Name::from("building")), Some(tree.at(2)));
    assert_eq!(tree.at(2).named(&Name::from("model")), None);
    assert_eq!(tree.at(0).named(&Name::from("graph")), Some(tree.at(0)));
    assert_eq!(Nest::WORKSPACE.named(&Name::from("graph")), None);
    assert_eq!(Some(<&[Name]>::from(tree.at(1))), tree.segments.get(..1));
}

#[test]
fn a_rooted_crate_has_no_name_and_never_equals_a_named_one() {
    let tree = Tree::new();
    let root = CrateRoot::new(PathBuf::from("crates/graph"));
    let rooted = Nest::module(Krate::Rooted(&root), &[]);
    assert_eq!(rooted.named(&Name::from("graph")), None);
    assert_ne!(rooted, tree.at(0));
    assert_eq!(rooted.to_string(), "crate");
    assert_eq!(Nest::module(Krate::Rooted(&root), &tree.segments).to_string(), "crate::model::building");
    assert_eq!(Krate::Named(&tree.krate).name(), Some(&tree.krate));
}
