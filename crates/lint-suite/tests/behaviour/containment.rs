use lint_core::{Lint, SourceFile};
use lint_fixtures::{Sources, Text};
use lint_transition_graph::{
    AcyclicQuotients, ClusterLint, ConnectedClusters, DeepClusters, InterfaceByConsumers, InterfaceByInternals,
    LowestCluster, OnClusters,
};

struct Clustered;

impl Clustered {
    fn lines<Check: ClusterLint>(sources: &[(&str, &str)]) -> Vec<String> {
        let files: Vec<SourceFile> = sources.iter().map(|(path, text)| Sources::file(*path, Text::new(text))).collect();
        OnClusters::<Check>::report(&files).to_string().lines().skip(1).map(str::to_owned).collect()
    }
}

const SPLIT: [(&str, &str); 4] = [
    ("crates/x/src/lib.rs", "mod a;\npub struct A(B);\npub struct B;\npub struct C(std::A);\n"),
    ("crates/x/src/a/mod.rs", "mod b;\nmod c;\npub struct D(E);\n"),
    ("crates/x/src/a/b.rs", "pub struct E;\n"),
    ("crates/x/src/a/c.rs", "pub struct F;\npub struct G;\n"),
];

#[test]
fn a_crate_or_directory_whose_types_fall_apart_is_reported_and_a_file_is_not() {
    assert_eq!(
        Clustered::lines::<ConnectedClusters>(&SPLIT),
        [
            "crates/x/src/a/b.rs:1: crate x falls into 5 groups: [A B] [C] [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
            "crates/x/src/a/b.rs:1: x::a falls into 3 groups: [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
        ]
    );
}

#[test]
fn a_linked_crate_is_not_reported() {
    let linked = [("crates/x/src/lib.rs", "mod a;\npub struct A(a::B);\n"), ("crates/x/src/a.rs", "pub struct B;\n")];
    assert_eq!(Clustered::lines::<ConnectedClusters>(&linked), Vec::<String>::new());
}

#[test]
fn modules_that_use_one_another_are_reported_with_the_closing_edges() {
    let siblings = [
        ("crates/x/src/lib.rs", "mod a;\nmod b;\n"),
        ("crates/x/src/a.rs", "pub struct A(crate::b::B);\npub struct C;\n"),
        ("crates/x/src/b.rs", "pub struct B(crate::a::C);\n"),
    ];
    let own = [("crates/x/src/lib.rs", "mod a;\npub struct Top(a::A);\n"), ("crates/x/src/a.rs", "pub struct A(crate::Top);\n")];
    let layered = [("crates/x/src/lib.rs", "mod a;\npub struct Top(a::A);\n"), ("crates/x/src/a.rs", "pub struct A;\n")];
    assert_eq!(
        Clustered::lines::<AcyclicQuotients>(&siblings),
        [
            "crates/x/src/a.rs:1: in crate x, the parts x::a, x::b use one another in a cycle, through x::a::A -> x::b::B, x::b::B -> x::a::C; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(
        Clustered::lines::<AcyclicQuotients>(&own),
        [
            "crates/x/src/a.rs:1: in crate x, the parts crate x own file, x::a use one another in a cycle, through x::a::A -> x::Top, x::Top -> x::a::A; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(Clustered::lines::<AcyclicQuotients>(&layered), Vec::<String>::new());
}

const USED: [(&str, &str); 2] = [
    ("crates/x/src/lib.rs", "pub struct P;\npub struct Q(S);\npub struct R(S);\nstruct S;\n"),
    ("crates/y/src/lib.rs", "pub struct U(x::P);\npub struct V(x::Q);\npub struct W(x::P, x::R);\n"),
];

#[test]
fn exports_that_no_outside_user_joins_are_reported() {
    assert_eq!(
        Clustered::lines::<InterfaceByConsumers>(&USED),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P R] [Q] that no outside user joins; split it along the groups, one part for each kind of user"
        ]
    );
}

#[test]
fn exports_that_share_no_private_item_are_reported() {
    assert_eq!(
        Clustered::lines::<InterfaceByInternals>(&USED),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P] [Q R] that no private item joins; split it along the groups, since they share no implementation"
        ]
    );
}

#[test]
fn a_type_outside_the_lowest_cluster_of_its_users_is_reported() {
    let placed = [
        ("crates/x/src/lib.rs", "mod a;\nmod b;\npub struct Helper;\npub struct Common;\n"),
        ("crates/x/src/a.rs", "pub struct User(crate::Helper);\npub struct Other(crate::Common);\npub struct Near(y::Far);\n"),
        ("crates/x/src/b.rs", "pub struct Third(crate::Common);\n"),
        ("crates/y/src/lib.rs", "pub struct Far;\n"),
    ];
    assert_eq!(
        Clustered::lines::<LowestCluster>(&placed),
        [
            "crates/x/src/lib.rs:3: x::Helper is used only within x::a; move it into the lowest cluster that holds all its users",
            "crates/y/src/lib.rs:1: y::Far is used only within x::a; move it into the lowest cluster that holds all its users",
        ]
    );
}

#[test]
fn a_cluster_with_fewer_exports_than_private_items_is_deep() {
    let deep = [
        ("crates/x/src/lib.rs", "pub struct Face(In1, In2);\nstruct In1;\nstruct In2;\n"),
        ("crates/y/src/lib.rs", "pub struct User(x::Face, z::Two);\n"),
        ("crates/z/src/lib.rs", "pub struct Two(Hid);\nstruct Hid;\n"),
    ];
    assert_eq!(
        Clustered::lines::<DeepClusters>(&deep),
        [
            "crates/x/src/lib.rs:1: crate x is deep: 1 exported, 2 private; nothing to fix: keep the interface this small as it grows"
        ]
    );
}
