use invariants::{Ignoring, ignored};
use lint_layout::UseEdges;

ignored! {
    enum Ignored {
        Unparsed => "text that does not parse as a use item has no edges, which the row then reports",
    }
}

struct Edges;

impl Edges {
    fn of(text: &str) -> Vec<String> {
        let edges = syntax::parse::parse_str::<syntax::ItemUse>(text).map(|item| {
            UseEdges::of_tree(&item.tree).iter().map(ToString::to_string).collect::<Vec<String>>()
        });
        edges.ignoring(Ignored::Unparsed).unwrap_or_default()
    }
}

#[test]
fn each_use_edge_is_the_path_before_its_leaf() {
    assert_eq!(Edges::of("use a::b;"), ["a"]);
    assert_eq!(Edges::of("use a::{b::c, d};"), ["a::b", "a"]);
    assert_eq!(Edges::of("use a::{b::{c, d::e}, f::*};"), ["a::b", "a::b::d", "a::f"]);
}
