use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_node_recursion::NodeRecursion;

const DIRECT: &str = "struct Walk;\nimpl Walk {\n    fn depth(expr: &syn::Expr) -> usize {\n        match expr {\n            syn::Expr::Paren(inner) => Self::depth(&inner.expr) + 1,\n            _ => 0,\n        }\n    }\n}\n";
const MUTUAL: &str = "fn outer(expr: &syn::Expr) -> usize {\n    inner(expr)\n}\nfn inner(expr: &syn::Expr) -> usize {\n    outer(expr)\n}\n";
const METHOD: &str =
    "struct Walk;\nimpl Walk {\n    fn seen(&self, expr: &syn::Expr) -> usize {\n        self.seen(expr)\n    }\n}\n";
const FLAT: &str = "struct Walk;\nimpl Walk {\n    fn depth(expr: &syn::Expr) -> usize {\n        Self::width(expr)\n    }\n    fn width(expr: &syn::Expr) -> usize {\n        0\n    }\n}\n";
const OTHER_TYPES: &str =
    "struct Tree;\nimpl Tree {\n    fn depth(tree: &Tree) -> usize {\n        Self::depth(tree)\n    }\n}\n";
const AMBIGUOUS: &str = "struct Name;\nimpl From<&syn::Ident> for Name {\n    fn from(ident: &syn::Ident) -> Name {\n        Name::from(String::new())\n    }\n}\nimpl From<String> for Name {\n    fn from(text: String) -> Name {\n        Name\n    }\n}\n";
const IN_SYNTAX: &str = "fn depth(expr: &syn::Expr) -> usize {\n    depth(expr)\n}\n";

const CASES: [Case; 7] = [
    Case::new(
        Title::new("a function over a node that calls itself"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(DIRECT))],
        Flagged::new(&["crates/a/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("two functions over nodes that call each other"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(MUTUAL))],
        Flagged::new(&["crates/a/src/lib.rs:1", "crates/a/src/lib.rs:4"]),
    ),
    Case::new(
        Title::new("a method over a node that calls itself on self"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(METHOD))],
        Flagged::new(&["crates/a/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("a node handed to another function that does not call back"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(FLAT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("recursion over a type of our own"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(OTHER_TYPES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a call that names more than one function of the type"),
        &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(AMBIGUOUS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("recursion inside the syntax crate"),
        &[Source::new(Located::new("crates/syntax/src/lib.rs"), Text::new(IN_SYNTAX))],
        Flagged::new(&[]),
    ),
];

#[test]
fn functions_over_syntax_nodes_do_not_recurse_outside_the_syntax_crate() {
    assert_eq!(Sources::mismatches::<Queried<NodeRecursion>>(&CASES), Lines::default());
}
