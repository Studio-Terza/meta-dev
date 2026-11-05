use lint_core::Violation;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::{
    AcyclicQuotients, Clustered as ClusterReport, ConnectedClusters, Extraction, GraphLint, InterfaceByConsumers,
    LowestCluster, TypeSystem,
};

struct Clustered;

impl Clustered {
    fn lines(
        sources: &[Source],
        report: impl for<'found, 'file> Fn(&TypeSystem<'found, 'file>) -> Vec<Violation<'file>>,
    ) -> Vec<String> {
        let files = Sources::parsed(sources);
        let extraction = Extraction::from(files.as_slice());
        let system = TypeSystem::built_from(&files, &extraction);
        report(&system).iter().map(ToString::to_string).collect()
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
        Clustered::lines(&SPLIT, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a/b.rs:1: crate x falls into 5 groups: [A B] [C] [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
            "crates/x/src/a/b.rs:1: x::a falls into 3 groups: [D E] [F] [G] with no link between them; split it along the groups, or link them through a type they share",
        ]
    );
}

#[test]
fn a_linked_crate_is_not_reported() {
    let linked = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct A(a::B);\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct B;\n"),
    ];
    assert_eq!(
        Clustered::lines(&linked, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn a_name_declared_in_two_modules_of_one_crate_resolves_by_its_own_module_and_its_imports() {
    let twins = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct Platform;\npub struct Leg(Platform);\n"),
        Clustered::at("crates/x/src/b.rs", "use crate::a::Leg;\npub struct Platform<'day>(&'day Leg);\n"),
        Clustered::at("crates/x/src/c.rs", "use crate::b::Platform;\npub struct User<'day>(Platform<'day>);\n"),
    ];
    assert_eq!(
        Clustered::lines(&twins, |view| ClusterReport::<AcyclicQuotients>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
    );
    assert_eq!(
        Clustered::lines(&twins, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a.rs:2: x::a::Leg is used only within x::b; move it into the lowest cluster that holds all its users",
            "crates/x/src/b.rs:2: x::b::Platform is used only within x::c; move it into the lowest cluster that holds all its users",
        ]
    );
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
        Clustered::lines(&siblings, |view| ClusterReport::<AcyclicQuotients>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a.rs:1: in crate x, the parts x::a, x::b use one another in a cycle, through x::a::A -> x::b::B, x::b::B -> x::a::C; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(
        Clustered::lines(&own, |view| ClusterReport::<AcyclicQuotients>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a.rs:1: in crate x, the parts crate x own file, x::a use one another in a cycle, through x::a::A -> x::Top, x::Top -> x::a::A; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
    assert_eq!(
        Clustered::lines(&layered, |view| ClusterReport::<AcyclicQuotients>::report_graph(view).violations().to_vec()),
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
        Clustered::lines(&USED, |view| ClusterReport::<InterfaceByConsumers>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: the exports of crate x fall into 2 groups: [P R] [Q] that no outside user joins; split it along the groups, one part for each kind of user"
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
        Clustered::lines(&placed, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/y/src/lib.rs:1: y::Far is used only within x::a; move it into the lowest cluster that holds all its users",
            "crates/x/src/lib.rs:3: x::Helper is used only within x::a; move it into the lowest cluster that holds all its users",
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
        Clustered::lines(&bodies, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
    );
    assert_eq!(
        Clustered::lines(&bodies, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
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
        Clustered::lines(&signed, |view| ClusterReport::<AcyclicQuotients>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
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
        .map(|sources| {
            Clustered::lines(sources, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec())
        })
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
        .map(|sources| {
            Clustered::lines(sources, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec())
        })
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

#[test]
fn an_imported_name_resolves_to_the_crate_it_is_imported_from() {
    let external = [
        Clustered::at("crates/x/src/lib.rs", "pub struct Command;\n"),
        Clustered::at("crates/y/src/lib.rs", "use std::process::Command;\npub struct Tool(Command);\n"),
    ];
    let workspace = [
        Clustered::at("crates/x/src/lib.rs", "pub struct P;\n"),
        Clustered::at("crates/y/src/lib.rs", "use x::P;\npub struct U(P);\n"),
        Clustered::at("crates/z/src/lib.rs", "pub struct P;\n"),
    ];
    let reported: Vec<Vec<String>> = [external.as_slice(), workspace.as_slice()]
        .iter()
        .map(|sources| {
            Clustered::lines(sources, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec())
        })
        .collect();
    assert_eq!(
        reported,
        [
            Vec::new(),
            vec![
                "crates/x/src/lib.rs:1: x::P is used only within crate y; move it into the lowest cluster that holds all its users"
                    .to_owned()
            ]
        ]
    );
}

#[test]
fn a_type_alias_stands_for_the_type_it_names() {
    let aliased = [
        Clustered::at("crates/x/src/lib.rs", "pub struct B;\npub type A = B;\n"),
        Clustered::at("crates/y/src/lib.rs", "pub struct U(x::A);\n"),
    ];
    assert_eq!(
        Clustered::lines(&aliased, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: x::B is used only within crate y; move it into the lowest cluster that holds all its users"
        ]
    );
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
        .map(|sources| {
            Clustered::lines(sources, |view| {
                ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()
            })
        })
        .collect();
    assert_eq!(reported, [Vec::<String>::new(), Vec::new()]);
}

#[test]
fn a_twin_resolves_through_self_super_and_child_module_paths() {
    let pathed = [
        Clustered::at(
            "crates/x/src/lib.rs",
            "mod a;\nmod b;\npub struct Platform;\npub struct Lounge(crate::a::Seat);\n",
        ),
        Clustered::at(
            "crates/x/src/a/mod.rs",
            "mod deep;\nmod inner;\npub struct Platform;\npub struct Bench(self::inner::Seat);\n",
        ),
        Clustered::at(
            "crates/x/src/a/inner.rs",
            "pub struct Uses(super::Platform);\npub struct Root(super::super::Platform);\npub struct Seat;\n",
        ),
        Clustered::at("crates/x/src/a/deep/mod.rs", "mod kid;\npub struct Holder(kid::Gauge);\n"),
        Clustered::at("crates/x/src/a/deep/kid.rs", "pub struct Gauge;\n"),
        Clustered::at("crates/x/src/b.rs", "pub struct Gauge;\npub struct Seat;\n"),
    ];
    assert_eq!(
        Clustered::lines(&pathed, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/lib.rs:3: x::Platform is used only within x::a::inner; move it into the lowest cluster that holds all its users",
            "crates/x/src/a/mod.rs:3: x::a::Platform is used only within x::a::inner; move it into the lowest cluster that holds all its users",
        ]
    );
    assert_eq!(
        Clustered::lines(&pathed, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a/deep/kid.rs:1: crate x falls into 6 groups: [Bench Lounge Seat] [Gauge Holder] [Gauge] [Platform Root] [Platform Uses] [Seat] with no link between them; split it along the groups, or link them through a type they share",
            "crates/x/src/a/deep/kid.rs:1: x::a falls into 4 groups: [Bench Seat] [Gauge Holder] [Platform Uses] [Root] with no link between them; split it along the groups, or link them through a type they share",
        ]
    );
}

#[test]
fn a_bare_twin_resolves_to_the_one_in_its_own_module() {
    let own = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\npub struct Cart;\n"),
        Clustered::at(
            "crates/x/src/a/mod.rs",
            "mod inner;\npub struct Cart;\npub struct Train(Cart);\npub struct Wagon;\nimpl Wagon {\n    fn cart(&self) -> crate::a::Cart {}\n}\n",
        ),
        Clustered::at("crates/x/src/a/inner.rs", "pub struct Seat(super::Cart);\n"),
    ];
    assert_eq!(
        Clustered::lines(&own, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/a/inner.rs:1: crate x falls into 2 groups: [Cart] [Cart Seat Train Wagon] with no link between them; split it along the groups, or link them through a type they share"
        ]
    );
}

#[test]
fn an_imported_workspace_type_wins_over_a_local_twin() {
    let imported = [
        Clustered::at("crates/x/src/lib.rs", "mod document;\nmod excerpt;\n"),
        Clustered::at("crates/x/src/document.rs", "mod text;\npub struct Document(text::Text);\n"),
        Clustered::at("crates/x/src/document/text.rs", "pub struct Text;\n"),
        Clustered::at("crates/x/src/excerpt.rs", "mod boundary;\npub struct Excerpt;\n"),
        Clustered::at(
            "crates/x/src/excerpt/boundary.rs",
            "use files::Text;\nuse super::Excerpt;\nimpl From<&Text> for Excerpt {\n    fn from(reason: &Text) -> Excerpt {\n        Excerpt::new(reason)\n    }\n}\n",
        ),
        Clustered::at("crates/files/src/lib.rs", "mod text;\npub use text::Text;\n"),
        Clustered::at("crates/files/src/text.rs", "pub struct Text;\n"),
    ];
    assert_eq!(
        Clustered::lines(&imported, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/files/src/text.rs:1: files::text::Text is used only within x::excerpt; move it into the lowest cluster that holds all its users"
        ]
    );
}

#[test]
fn a_variant_path_names_its_enum_and_a_module_path_names_its_type() {
    let variants = [
        Clustered::at("crates/x/src/lib.rs", "mod caller;\nmod event;\nmod hub;\nmod rule;\n"),
        Clustered::at(
            "crates/x/src/caller.rs",
            "pub struct Caller;\nimpl Caller {\n    fn pull(&self) {\n        crate::hub::pull_request::PullRequest::new();\n    }\n}\n",
        ),
        Clustered::at(
            "crates/x/src/event.rs",
            "pub enum Event {\n    PullRequest,\n    Other,\n}\nimpl Event {\n    fn other(&self) -> bool {\n        matches!(self, Self::Other)\n    }\n}\n",
        ),
        Clustered::at(
            "crates/x/src/hub.rs",
            "mod pull_request;\nuse crate::event::Event;\npub struct Hub;\nimpl Hub {\n    fn pulled(&self) {\n        Event::PullRequest;\n    }\n}\n",
        ),
        Clustered::at("crates/x/src/hub/pull_request.rs", "pub struct PullRequest;\n"),
        Clustered::at("crates/x/src/rule.rs", "pub enum Rule {\n    Linear(u8),\n    Plain { width: u8 },\n}\n"),
        Clustered::at("crates/x/src/linear.rs", "pub struct Linear;\npub struct Plain;\n"),
    ];
    assert_eq!(
        Clustered::lines(&variants, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/event.rs:1: x::event::Event is used only within x::hub; move it into the lowest cluster that holds all its users",
            "crates/x/src/hub/pull_request.rs:1: x::hub::pull_request::PullRequest is used only within x::caller; move it into the lowest cluster that holds all its users",
        ]
    );
    assert_eq!(
        Clustered::lines(&variants, |view| ClusterReport::<InterfaceByConsumers>::report_graph(view)
            .violations()
            .to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn a_type_parameter_is_not_a_use_of_a_type_with_its_name() {
    let generic = [
        Clustered::at("crates/x/src/lib.rs", "mod layer;\npub struct Look;\n"),
        Clustered::at(
            "crates/x/src/layer.rs",
            "pub struct Layer<Look> {\n    features: Vec<Look>,\n}\nimpl<Look: Default> Layer<Look> {\n    fn first(&self) -> Look {\n        Look::default()\n    }\n}\n",
        ),
    ];
    assert_eq!(
        Clustered::lines(&generic, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn a_trait_impl_links_the_twin_its_own_file_names() {
    let twins = [
        Clustered::at("crates/x/src/lib.rs", "mod check;\nmod readout;\n"),
        Clustered::at("crates/x/src/check.rs", "mod geometry;\nmod rule;\npub struct Check(geometry::Geometry);\n"),
        Clustered::at(
            "crates/x/src/check/geometry.rs",
            "use super::rule::Rule;\npub struct Geometry;\nimpl Rule for Geometry {}\n",
        ),
        Clustered::at("crates/x/src/check/rule.rs", "pub trait Rule {}\n"),
        Clustered::at("crates/x/src/readout.rs", "mod geometry;\npub struct Readout(geometry::Geometry);\n"),
        Clustered::at("crates/x/src/readout/geometry.rs", "pub struct Geometry;\n"),
    ];
    assert_eq!(
        Clustered::lines(&twins, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/check/rule.rs:1: x::check::rule::Rule is used only within x::check::geometry; move it into the lowest cluster that holds all its users"
        ]
    );
}

#[test]
fn a_body_use_links_across_crates_through_its_import_or_qualifier() {
    let imported = [
        Clustered::at("crates/x/src/lib.rs", "pub struct Mark;\npub struct Touch;\npub struct Loose;\n"),
        Clustered::at(
            "crates/y/src/lib.rs",
            "use x::Mark;\npub struct View;\nimpl View {\n    fn draw(&self) {\n        Mark::new();\n        x::Touch::new();\n        Loose::new();\n    }\n}\n",
        ),
    ];
    assert_eq!(
        Clustered::lines(&imported, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: x::Mark is used only within crate y; move it into the lowest cluster that holds all its users",
            "crates/x/src/lib.rs:2: x::Touch is used only within crate y; move it into the lowest cluster that holds all its users",
        ]
    );
}

macro_rules! anchor_report_for_macro_string {
    ($literal:literal) => {
        Clustered::lines(
            &[
                Clustered::at("crates/x/src/lib.rs", "mod view;\npub struct Anchor;\n"),
                Clustered::at(
                    "crates/x/src/view.rs",
                    concat!(
                        "use crate::Anchor;\npub struct View;\nimpl View {\n    fn draw(&self) {\n        rsx! { section { id: \"",
                        $literal,
                        "\" } };\n    }\n}\n"
                    ),
                ),
            ],
            |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec(),
        )
    };
}

#[test]
fn a_type_named_in_an_interpolated_macro_string_is_used() {
    assert_eq!(
        anchor_report_for_macro_string!("{Anchor::new(1)} {{plain}}"),
        [
            "crates/x/src/lib.rs:2: x::Anchor is used only within x::view; move it into the lowest cluster that holds all its users"
        ]
    );
}

#[test]
fn a_type_named_in_escaped_braces_of_a_macro_string_is_not_used() {
    assert_eq!(anchor_report_for_macro_string!("{{Anchor::new(1)}}"), Vec::<String>::new());
}

#[test]
fn a_const_standing_for_a_variant_stands_for_its_enum() {
    let constant = [
        Clustered::at("crates/x/src/lib.rs", "mod fast;\nmod mode;\nmod user;\n"),
        Clustered::at("crates/x/src/fast.rs", "pub struct Fast;\n"),
        Clustered::at("crates/x/src/mode.rs", "pub enum Mode {\n    Fast,\n}\n"),
        Clustered::at(
            "crates/x/src/user.rs",
            "use crate::mode::Mode;\nconst DEFAULT: Mode = Mode::Fast;\npub struct User;\nimpl User {\n    fn mode(&self) {\n        DEFAULT;\n    }\n}\n",
        ),
    ];
    assert_eq!(
        Clustered::lines(&constant, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/mode.rs:1: x::mode::Mode is used only within x::user; move it into the lowest cluster that holds all its users"
        ]
    );
}

#[test]
fn a_const_through_a_foreign_enum_stands_for_no_local_variant_twin() {
    let foreign = [
        Clustered::at("crates/x/src/lib.rs", "mod fast;\nmod user;\n"),
        Clustered::at("crates/x/src/fast.rs", "pub struct Fast;\n"),
        Clustered::at(
            "crates/x/src/user.rs",
            "use y::Mode;\nconst DEFAULT: Mode = Mode::Fast;\npub struct User;\nimpl User {\n    fn mode(&self) {\n        DEFAULT;\n    }\n}\n",
        ),
        Clustered::at("crates/y/src/lib.rs", "pub enum Mode {\n    Fast,\n}\n"),
    ];
    assert_eq!(
        Clustered::lines(&foreign, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        Vec::<String>::new()
    );
}

#[test]
fn a_body_path_reads_through_a_renamed_import_and_keeps_its_module_qualifier() {
    let renamed = [
        Clustered::at("crates/x/src/lib.rs", "pub struct Mark;\n"),
        Clustered::at("crates/y/src/lib.rs", "mod fast;\nmod view;\n"),
        Clustered::at("crates/y/src/fast.rs", "pub struct Fast;\n"),
        Clustered::at(
            "crates/y/src/view.rs",
            "mod inner;\nuse x::Mark as Badge;\nuse crate::fast::Fast;\npub struct View;\nimpl View {\n    fn draw(&self) {\n        Badge::new();\n        inner::Fast::new();\n    }\n}\n",
        ),
        Clustered::at("crates/y/src/view/inner.rs", "pub struct Fast;\n"),
    ];
    assert_eq!(
        Clustered::lines(&renamed, |view| ClusterReport::<LowestCluster>::report_graph(view).violations().to_vec()),
        [
            "crates/x/src/lib.rs:1: x::Mark is used only within y::view; move it into the lowest cluster that holds all its users"
        ]
    );
    assert_eq!(
        Clustered::lines(&renamed, |view| ClusterReport::<ConnectedClusters>::report_graph(view).violations().to_vec()),
        [
            "crates/y/src/fast.rs:1: crate y falls into 2 groups: [Fast] [Fast View] with no link between them; split it along the groups, or link them through a type they share"
        ]
    );
}

#[test]
fn field_and_variant_names_are_not_references_but_their_values_are() {
    let declared = [
        Clustered::at(
            "crates/x/src/lib.rs",
            "mod a;\npub struct B;\nimpl B {\n    const VALUE: isize = 1;\n}\nconst LIMIT: isize = B::VALUE;\nmacro_rules! made {\n    () => {\n        B\n    };\n}\n",
        ),
        Clustered::at(
            "crates/x/src/a.rs",
            "pub struct S {\n    made: u8,\n}\npub enum E {\n    V = crate::LIMIT,\n}\n",
        ),
    ];
    assert_eq!(
        Clustered::lines(&declared, |view| ClusterReport::<ConnectedClusters>::report_graph(view)
            .violations()
            .to_vec()),
        [
            "crates/x/src/a.rs:1: crate x falls into 2 groups: [B E] [S] with no link between them; split it along the groups, or link them through a type they share"
        ]
    );
}

#[test]
fn a_cycle_closed_through_a_trait_impl_is_reported() {
    let implemented = [
        Clustered::at("crates/x/src/lib.rs", "mod a;\nmod b;\n"),
        Clustered::at("crates/x/src/a.rs", "pub struct A;\nimpl crate::b::Tr for A {}\n"),
        Clustered::at("crates/x/src/b.rs", "pub trait Tr {}\npub struct B(crate::a::A);\n"),
    ];
    assert_eq!(
        Clustered::lines(&implemented, |view| ClusterReport::<AcyclicQuotients>::report_graph(view)
            .violations()
            .to_vec()),
        [
            "crates/x/src/a.rs:1: in crate x, the parts x::a, x::b use one another in a cycle, through x::a::A -> x::b::Tr, x::b::B -> x::a::A; move the shared types into one part that the others use, so the parts form layers"
        ]
    );
}
