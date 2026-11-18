use lint_fixtures::{Checks, Located, Shown, Source, Sources, Text};
use lint_transition_graph::UndeclaredAlgebra;

use crate::shown::OnGraph;

const ALGEBRA: &str =
    "pub trait Semigroup {\n    fn combine(self, other: Self) -> Self;\n}\npub trait Monoid: Semigroup + Default {}\n";

const SHAPES: &str = "use algebra::{Monoid, Semigroup};\npub struct A;\npub struct B;\nimpl A {\n    pub fn refined(self) -> A {}\n    pub fn with(self, b: B) -> A {}\n    pub fn joined(self, other: A) -> A {}\n    pub fn push(&mut self, b: B) {}\n    pub fn fill(out: &mut Vec<B>) {}\n    pub fn moved(self) -> B {}\n}\npub struct Sum;\nimpl Semigroup for Sum {\n    fn combine(self, other: Sum) -> Sum {}\n}\nimpl Monoid for Sum {}\nimpl std::fmt::Display for A {\n    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}\n}\nmod store {\n    impl Records {\n        fn insert(&mut self, record: super::A) {}\n    }\n}\n";

const DISGUISED: &str = "use std::ops::Add;\nuse algebra::{Monoid, Semigroup};\npub struct Plain;\nimpl Add for Plain {\n    type Output = Plain;\n    fn add(self, other: Plain) -> Plain {}\n}\npub struct Lawful;\nimpl Add for Lawful {\n    type Output = Lawful;\n    fn add(self, other: Lawful) -> Lawful {}\n}\nimpl Semigroup for Lawful {\n    fn combine(self, other: Lawful) -> Lawful {}\n}\npub struct Bag;\nimpl Extend<Bag> for Bag {\n    fn extend<Items: IntoIterator<Item = Bag>>(&mut self, items: Items) {}\n}\nimpl FromIterator<Self> for Bag {\n    fn from_iter<Items: IntoIterator<Item = Bag>>(items: Items) -> Bag {}\n}\npub struct Pile;\nimpl Extend<Pile> for Pile {\n    fn extend<Items: IntoIterator<Item = Pile>>(&mut self, items: Items) {}\n}\nimpl Semigroup for Pile {\n    fn combine(self, other: Pile) -> Pile {}\n}\nimpl Monoid for Pile {}\npub struct Other;\nimpl Add<u8> for Other {\n    type Output = u8;\n    fn add(self, other: u8) -> u8 {}\n}\n";

const NAMESAKE: &str = "pub trait Semigroup {\n    fn combine(self, other: Self) -> Self;\n}\npub struct C;\nimpl Semigroup for C {\n    fn combine(self, other: C) -> C {}\n}\n";

const LIBRARY: Source = Source::new(Located::new("crates/algebra/src/lib.rs"), Text::new(ALGEBRA));
const WORKSPACE_FIX: &str =
    "remove each site with the fix its own finding names; the lint blocks once the workspace count reaches zero";
const HIDDEN_FIX: &str = "declare the algebra this impl hides: implement algebra::Semigroup, and algebra::Monoid when there is an identity, for the type";

struct Fixture;

impl Fixture {
    fn reported(sources: &[Source]) -> Vec<String> {
        <OnGraph<UndeclaredAlgebra> as Checks>::reported(&Sources::parsed(sources), Shown::listed)
    }

    fn hidden(line: &str) -> String {
        format!("{line}; {HIDDEN_FIX}")
    }

    fn workspace(line: &str) -> String {
        format!("{line}; {WORKSPACE_FIX}")
    }
}

laws::table!(
    every_undeclared_same_type_edge_is_reported_with_its_shape_and_a_declared_semigroup_combine_is_not,
    Fixture::reported,
    [
        [LIBRARY, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SHAPES))] => vec![
            "crates/x/src/lib.rs:5: x::A::refined is an undeclared unary T -> T; write the refinement type this produces (transition-graph D14), so the edge goes from the type to a new type".to_owned(),
            "crates/x/src/lib.rs:6: x::A::with is an undeclared builder (T, X) -> T; make each fixed step a typestate edge T<S1> to T<S2>, or turn a per-item accumulation into contributions merged by a declared algebra::Monoid".to_owned(),
            "crates/x/src/lib.rs:7: x::A::joined is an undeclared binary (T, T) -> T; implement algebra::Semigroup for the type and make this its combine".to_owned(),
            "crates/x/src/lib.rs:8: x::A::push is an undeclared mutating &mut; return a new value, or move the mutation into a store operation".to_owned(),
            "crates/x/src/lib.rs:9: x::A::fill is an undeclared mutating &mut; return a new value, or move the mutation into a store operation".to_owned(),
            Fixture::workspace("crates/x/src/lib.rs:5: workspace: 5 undeclared same-type edges, 1 unary, 1 builder, 1 binary, 2 mutating; 0 std impls are an undeclared algebra; 1 edges with a signature fixed by a foreign trait are counted apart"),
        ],
    ]
);

laws::table!(
    std_impls_that_hide_an_algebra_are_reported_until_the_algebra_is_declared,
    Fixture::reported,
    [
        [LIBRARY, Source::new(Located::new("crates/x/src/lib.rs"), Text::new(DISGUISED))] => vec![
            Fixture::hidden("crates/x/src/lib.rs:4: Plain implements std::ops::Add with Output = Self without algebra::Semigroup"),
            Fixture::hidden("crates/x/src/lib.rs:17: Bag implements Extend<Self> without algebra::Monoid"),
            Fixture::hidden("crates/x/src/lib.rs:20: Bag implements FromIterator<Self> without algebra::Monoid"),
            Fixture::workspace("crates/x/src/lib.rs:4: workspace: 0 undeclared same-type edges, 0 unary, 0 builder, 0 binary, 0 mutating; 3 std impls are an undeclared algebra; 4 edges with a signature fixed by a foreign trait are counted apart"),
        ],
    ]
);

laws::table!(
    a_foreign_trait_named_semigroup_declares_nothing,
    Fixture::reported,
    [
        [LIBRARY, Source::new(Located::new("crates/y/src/lib.rs"), Text::new(NAMESAKE))] => vec![
            "crates/y/src/lib.rs:6: y::C::combine is an undeclared binary (T, T) -> T; implement algebra::Semigroup for the type and make this its combine".to_owned(),
            Fixture::workspace("crates/y/src/lib.rs:6: workspace: 1 undeclared same-type edges, 0 unary, 0 builder, 1 binary, 0 mutating; 0 std impls are an undeclared algebra; 0 edges with a signature fixed by a foreign trait are counted apart"),
        ],
    ]
);
