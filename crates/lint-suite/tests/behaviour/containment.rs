use lint_core::Violation;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::{
    AcyclicQuotients, ConnectedClusters, Containment, DeepClusters, InterfaceByConsumers, InterfaceByInternals,
    LowestCluster,
};
use transition_facts::Transitions;
use type_facts::Facts;

struct Clustered;

impl Clustered {
    fn lines(
        sources: &[Source],
        report: impl for<'found, 'file> Fn(&Containment<'found, 'file>) -> Vec<Violation<'file>>,
    ) -> Vec<String> {
        let files = Sources::parsed(sources);
        let facts = Facts::of_files(&files);
        let transitions = Transitions::of_files(&files);
        let view = Containment::clustered(&files, &facts, &transitions);
        report(&view).iter().map(ToString::to_string).collect()
    }

    const fn at(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }
}

const SPLIT: [Source; 4] = [
    Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct A(B);\npub struct B;\npub struct C(std::A);\n"),
    Clustered::at("crates/x/src/a/mod.rs", "mod b;\nmod c;\npub struct D(E);\n"),
    Clustered::at("crates/x/src/a/b.rs", "pub struct E;\n"),
    Clustered::at("crates/x/src/a/c.rs", "pub struct F;\npub struct G;\n"),
];

#[test]
fn a_crate_or_directory_whose_types_fall_apart_is_reported_and_a_file_is_not() {
    assert_eq!(
        Clustered::lines(&SPLIT, |view| view.report::<ConnectedClusters>().violations().to_vec()),
        [
            "crates/x/src/a/b.rs:1: crate x falls into 5 groups: [A B] [C] [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
            "crates/x/src/a/b.rs:1: x::a falls into 3 groups: [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
        ]
    );
}

#[test]
fn a_linked_crate_is_not_reported_and_a_name_declared_twice_in_it_is_left_out() {
    let linked = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct A(a::B);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct B;\n"),
    ];
    let twins = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct T;\npub struct U(T);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct T;\n"),
    ];
    let reported: Vec<Vec<String>> = [linked, twins]
        .iter()
        .map(|sources| Clustered::lines(sources, |view| view.report::<ConnectedClusters>().violations().to_vec()))
        .collect();
    assert_eq!(reported, [Vec::<String>::new(), Vec::new()]);
}

#[test]
fn modules_that_use_one_another_are_reported_with_the_closing_edges() {
    let siblings = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\nmod b;\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct A(crate::b::B);\npub struct C;\n"),
        Clustered::at("crates/x/src/b.rs", "pub struct B(crate::a::C);\n"),
    ];
    let own = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct Top(a::A);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct A(crate::Top);\n"),
    ];
    let layered = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct Top(a::A);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct A;\n"),
    ];
    assert_eq!(
        Clustered::lines(&siblings, |view| view.report::<AcyclicQuotients>().violations().to_vec()),
        [
            "crates/x/src/a.rs:1: in crate x, the parts x::a, x::b use one another in a cycle, through x::a::A -> x::b::B, x::b::B -> x::a::C; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(
        Clustered::lines(&own, |view| view.report::<AcyclicQuotients>().violations().to_vec()),
        [
            "crates/x/src/a.rs:1: in crate x, the parts crate x own file, x::a use one another in a cycle, through x::a::A -> x::Top, x::Top -> x::a::A; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(
        Clustered::lines(&layered, |view| view.report::<AcyclicQuotients>().violations().to_vec()),
        Vec::<String>::new()
    );
}

const USED: [Source; 2] = [
    Clustered::at("crates/x/src/lib.rs", "pub struct P;\npub struct Q(S);\npub struct R(S);\nstruct S;\n"),
    Clustered::at("crates/y/src/lib.rs", "pub struct U(x::P);\npub struct V(x::Q);\npub struct W(x::P, x::R);\n"),
];

#[test]
fn exports_that_no_outside_user_joins_are_reported() {
    assert_eq!(
        Clustered::lines(&USED, |view| view.report::<InterfaceByConsumers>().violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P R] [Q] that no outside user joins; split it along the groups, one part for each kind of user"
        ]
    );
}

#[test]
fn exports_that_share_no_private_item_are_reported() {
    assert_eq!(
        Clustered::lines(&USED, |view| view.report::<InterfaceByInternals>().violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P] [Q R] that no private item joins; split it along the groups, since they share no implementation"
        ]
    );
}

#[test]
fn a_type_outside_the_lowest_cluster_of_its_users_is_reported() {
    let placed = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\nmod b;\npub struct Helper;\npub struct Common;\n"),
        Clustered::at(
            "crates/x/src/a.rs",
            "pub struct User(crate::Helper);\npub struct Other(crate::Common);\npub struct Near(y::Far);\n",
        ),
        Clustered::at("crates/x/src/b.rs", "pub struct Third(crate::Common);\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct Far;\n"),
    ];
    assert_eq!(
        Clustered::lines(&placed, |view| view.report::<LowestCluster>().violations().to_vec()),
        [
            "crates/y/src/lib.rs:1: y::Far is used only within x::a; move it into the lowest cluster that holds all its users",
            "crates/x/src/lib.rs:3: x::Helper is used only within x::a; move it into the lowest cluster that holds all its users",
        ]
    );
}

#[test]
fn a_cluster_with_fewer_exports_than_private_items_is_deep() {
    let deep = [
        Clustered::at("crates/x/src/lib.rs", "pub struct Face(In1, In2);\nstruct In1;\nstruct In2;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct User(x::Face, z::Two);\n"),
        Clustered::at("crates/z/src/lib.rs", "pub struct Two(Hid);\nstruct Hid;\n"),
    ];
    assert_eq!(
        Clustered::lines(&deep, |view| view.report::<DeepClusters>().violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: crate x is deep: 1 exported, 2 private; nothing to fix: keep the interface this small as it grows"
        ]
    );
}

#[test]
fn a_type_named_only_in_a_method_body_is_linked_within_its_crate() {
    let bodies = [
        Clustered::at(
            "crates/x/src/lib.rs",
            "pub struct A;\npub struct B;\nimpl A {\n    fn f(&self) {\n        B::new();\n        Far::new();\n    }\n}\n",
        ),
        Clustered::at("crates/y/src/lib.rs", "pub struct Far;\n"),
    ];
    assert_eq!(
        Clustered::lines(&bodies, |view| view.report::<ConnectedClusters>().violations().to_vec()),
        Vec::<String>::new()
    );
    assert_eq!(
        Clustered::lines(&bodies, |view| view.report::<LowestCluster>().violations().to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn a_cycle_closed_only_through_a_signature_is_not_reported() {
    let signed = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\nmod b;\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct A;\nimpl A {\n    fn b(&self) -> crate::b::B {}\n}\n"),
        Clustered::at("crates/x/src/b.rs", "pub struct B(crate::a::A);\n"),
    ];
    assert_eq!(
        Clustered::lines(&signed, |view| view.report::<AcyclicQuotients>().violations().to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn exports_that_reach_only_one_another_share_no_private_item() {
    let chained = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P(Q);\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct U(x::P);\npub struct V(x::Q);\n"),
    ];
    assert_eq!(
        Clustered::lines(&chained, |view| view.report::<InterfaceByInternals>().violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P] [Q] that no private item joins; split it along the groups, since they share no implementation"
        ]
    );
}

#[test]
fn a_name_declared_in_two_crates_resolves_to_its_qualifier_else_to_the_own_crate() {
    let qualified = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct P;\npub struct U(x::P);\n"),
    ];
    let bare = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct P;\npub struct W(P);\n"),
    ];
    let reported: Vec<Vec<String>> = [qualified, bare]
        .iter()
        .map(|sources| Clustered::lines(sources, |view| view.report::<LowestCluster>().violations().to_vec()))
        .collect();
    assert_eq!(
        reported,
        [
            vec![
                "crates/x/src/lib.rs:1: x::P is used only within crate y; move it into the lowest cluster that holds all its users"
                    .to_owned()
            ],
            Vec::new()
        ]
    );
}

#[test]
fn a_field_with_an_unqualified_name_and_a_method_signature_both_link_across_crates() {
    let apart = [
        Clustered::at("crates/x/src/lib.rs", "pub struct W;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct V(W);\n"),
    ];
    let signed = [
        Clustered::at("crates/x/src/lib.rs", "pub struct B;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct A;\nimpl A {\n    fn b(&self) -> x::B {}\n}\n"),
    ];
    let reported: Vec<Vec<String>> = [apart, signed]
        .iter()
        .map(|sources| Clustered::lines(sources, |view| view.report::<LowestCluster>().violations().to_vec()))
        .collect();
    assert_eq!(
        reported,
        [
            [
                "crates/x/src/lib.rs:1: x::W is used only within crate y; move it into the lowest cluster that holds all its users"
            ],
            [
                "crates/x/src/lib.rs:1: x::B is used only within crate y; move it into the lowest cluster that holds all its users"
            ]
        ]
    );
}
