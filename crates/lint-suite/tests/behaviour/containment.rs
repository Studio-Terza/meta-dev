use lint_core::Violation;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::{
    AcyclicQuotients, ConnectedClusters, ConnectedDirectories, Containment, CrateInterfaces, InterfaceByConsumers,
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
fn a_crate_or_directory_whose_types_fall_apart_is_flagged_and_a_file_is_not() {
    assert_eq!(
        Clustered::lines(&SPLIT, |view| view.report::<ConnectedClusters>().violations().to_vec()),
        [
            "crates/x/src/a/b.rs:1: crate x falls into 5 groups: [A B] [C] [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
        ]
    );
    assert_eq!(
        Clustered::lines(&SPLIT, |view| view.report::<ConnectedDirectories>().violations().to_vec()),
        [
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
        Clustered::lines(&USED, |view| view.report::<CrateInterfaces>().violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P R] [Q] that no outside user joins; split it along the groups, one part for each kind of user"
        ]
    );
    let modules = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct U(a::P);\npub struct V(a::Q);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct P;\npub struct Q;\n"),
    ];
    assert_eq!(
        Clustered::lines(&modules, |view| view.report::<InterfaceByConsumers>().violations().to_vec()),
        [
            "crates/x/src/a.rs:1: the exports of x::a fall into 2 groups: [P] [Q] that no outside user joins; split it along the groups, one part for each kind of user"
        ]
    );
}

#[test]
fn a_type_outside_the_lowest_module_of_its_users_in_its_crate_is_flagged() {
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
            "crates/x/src/lib.rs:3: x::Helper is used only within x::a; move it into the lowest module that holds all its users",
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

const SECOND_USER: Source = Clustered::at("crates/v/src/lib.rs", "pub struct V(x::Q);\n");

impl Clustered {
    fn interfaces(sources: &[Source]) -> Vec<String> {
        Clustered::lines(sources, |view| view.report::<CrateInterfaces>().violations().to_vec())
    }

    fn split(first: &str) -> Vec<String> {
        vec![format!(
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: {first} that no outside user joins; split it along the groups, one part for each kind of user"
        )]
    }
}

#[test]
fn a_name_declared_in_two_crates_resolves_to_its_qualifier_else_to_the_own_crate() {
    let qualified = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct P;\npub struct U(x::P);\n"),
        SECOND_USER,
    ];
    let bare = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct P;\npub struct W(P);\n"),
        SECOND_USER,
    ];
    assert_eq!(
        [Clustered::interfaces(&qualified), Clustered::interfaces(&bare)],
        [Clustered::split("[P] [Q]"), Vec::new()]
    );
}

#[test]
fn a_field_with_an_unqualified_name_and_a_method_signature_both_link_across_crates() {
    let apart = [
        Clustered::at("crates/x/src/lib.rs", "pub struct W;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct U(W);\n"),
        SECOND_USER,
    ];
    let signed = [
        Clustered::at("crates/x/src/lib.rs", "pub struct B;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct A;\nimpl A {\n    fn b(&self) -> x::B {}\n}\n"),
        SECOND_USER,
    ];
    assert_eq!(
        [Clustered::interfaces(&apart), Clustered::interfaces(&signed)],
        [Clustered::split("[Q] [W]"), Clustered::split("[B] [Q]")]
    );
}

#[test]
fn an_imported_name_resolves_to_the_crate_it_is_imported_from() {
    let external = [
        Clustered::at("crates/x/src/lib.rs", "pub struct Command;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "use std::process::Command;\npub struct Tool(Command);\n"),
        SECOND_USER,
    ];
    let workspace = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "use x::P;\npub struct U(P);\n"),
        Clustered::at("crates/z/src/lib.rs", "pub struct P;\n"),
        SECOND_USER,
    ];
    assert_eq!(
        [Clustered::interfaces(&external), Clustered::interfaces(&workspace)],
        [Vec::new(), Clustered::split("[P] [Q]")]
    );
}

#[test]
fn a_type_alias_stands_for_the_type_it_names() {
    let aliased = [
        Clustered::at("crates/x/src/lib.rs", "pub struct B;\npub type A = B;\npub struct Q;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct U(x::A);\n"),
        SECOND_USER,
    ];
    assert_eq!(Clustered::interfaces(&aliased), Clustered::split("[B] [Q]"));
}

#[test]
fn a_const_or_a_macro_used_in_a_body_links_the_types_it_names() {
    let constant = [Clustered::at(
        "crates/x/src/lib.rs",
        "pub struct A;\npub struct B;\nconst LIMIT: B = B;\nimpl A {\n    fn f(&self) {\n        LIMIT;\n    }\n}\n",
    )];
    let macro_rules = [Clustered::at(
        "crates/x/src/lib.rs",
        "pub struct A;\npub struct B;\nmacro_rules! made {\n    () => {\n        B\n    };\n}\nimpl A {\n    fn f(&self) {\n        made!();\n    }\n}\n",
    )];
    let reported: Vec<Vec<String>> = [constant, macro_rules]
        .iter()
        .map(|sources| Clustered::lines(sources, |view| view.report::<ConnectedClusters>().violations().to_vec()))
        .collect();
    assert_eq!(reported, [Vec::<String>::new(), Vec::new()]);
}
