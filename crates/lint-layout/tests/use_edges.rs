use lint_layout::UseEdges;

laws::table!(each_use_edge_is_the_path_before_its_leaf, |text: &&str| {
    let edges = syn::parse_str::<syn::ItemUse>(text).map(|item| {
        UseEdges::of_tree(&item.tree).iter().map(ToString::to_string).collect::<Vec<String>>()
    });
    edges.unwrap_or_default()
}, [
    "use a::b;" => ["a"],
    "use a::{b::c, d};" => ["a::b", "a"],
    "use a::{b::{c, d::e}, f::*};" => ["a::b", "a::b::d", "a::f"],
]);
