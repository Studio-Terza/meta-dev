use lint_layout::Name;
use lint_placement::{Level, Nest};

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

    fn at(&self, depth: usize) -> Nest<'_, &Name> {
        Nest::module(&self.krate, self.segments.get(..depth).unwrap_or(&self.segments))
    }
}

#[test]
fn a_module_climbs_through_its_crate_to_the_workspace() {
    let tree = Tree::new();
    let lineage: Vec<String> = tree.at(2).lineage().map(|nest| nest.to_string()).collect();
    assert_eq!(lineage, ["graph::model::building", "graph::model", "crate graph", "the workspace"]);
    assert_eq!(Nest::<&Name>::WORKSPACE.parent(), None);
}

#[test]
fn each_place_knows_its_level() {
    let tree = Tree::new();
    assert_eq!(
        [tree.at(2).level(), tree.at(0).level(), Nest::<&Name>::WORKSPACE.level()],
        [Level::Module, Level::Crate, Level::Workspace]
    );
}

#[test]
fn an_outer_place_holds_its_inner_ones_and_not_its_neighbours() {
    let tree = Tree::new();
    let other = Nest::module(&tree.other, &tree.segments);
    assert_eq!(tree.at(1).holding(tree.at(2)), Some(tree.at(1)));
    assert_eq!(tree.at(2).holding(tree.at(1)), None);
    assert_eq!(tree.at(1).holding(other), None);
    assert_eq!(tree.at(2).common(other), Nest::WORKSPACE);
    assert_eq!(tree.at(2).common(tree.at(1)), tree.at(1));
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
