use lint_layout::{UseEdges, UsePath};

laws::table!(each_use_edge_is_the_path_before_its_leaf, |text: &&str| {
    let edges = syn::parse_file(text).map(|file| {
        let mut seen = Vec::new();
        UseEdges::each(&file, |path: &UsePath| seen.push(path.to_string()));
        seen
    });
    edges.unwrap_or_default()
}, [
    "use a::b;" => ["a"],
    "use a::{b::c, d};" => ["a::b", "a"],
    "use a::{b::{c, d::e}, f::*};" => ["a::b", "a::b::d", "a::f"],
]);
