use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::AlgebraLaws;

use crate::shown::OnGraph;

struct Declared;

impl Declared {
    fn of(sources: &[Source]) -> String {
        OnGraph::<AlgebraLaws>::report(&Sources::parsed(sources)).to_string()
    }

    fn lawed(source: &'static str, law: &'static str) -> String {
        Self::of(&[Self::at("crates/x/src/lib.rs", source), Self::at("crates/x/tests/t.rs", law)])
    }

    fn at(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }
}

const ADD: &str = "struct A;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A {}\n}\n";
const MANY: &str = "struct A;\nstruct B;\nstruct Factor;\nimpl A {\n    fn zero() -> A {}\n    fn scale(&self, factor: Factor) -> A {}\n    fn get(&self) -> Option<A> {}\n    fn bump(&mut self) {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";
const CLONED: &str = "struct A;\nimpl Clone for A {\n    fn clone(&self) -> A {}\n}\nimpl PartialEq for A {\n    fn eq(&self, other: &A) -> bool {}\n}\n";
const MULTIPLIED: &str = "struct A;\nimpl Mul<f64> for A {\n    type Output = A;\n    fn mul(self, rhs: f64) -> A {}\n}\nimpl Neg for A {\n    type Output = A;\n    fn neg(self) -> A {}\n}\n";
const TRANSITION: &str = "struct A;\nstruct B;\nimpl A {\n    fn into_b(&self) -> B {}\n}\n";
const TESTED: &str =
    "struct A;\n#[cfg(test)]\nmod tests {\n    impl A {\n        fn join(self, other: A) -> A {}\n    }\n}\n";
const PRIVATE: &str = "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\n";
const PUBLIC: &str = "pub struct A;\nimpl A {\n    pub fn join(self, other: A) -> A {}\n}\n";
const CLEARING: &str = "laws::associative!(add_regroups, any::<A>(), A::add);\n";
const COMMUTING: &str = "laws::commutative!(add_reorders, any::<A>(), A::add);\n";
const IDEMPOTENT: &str = "laws::idempotent!(add_settles, any::<A>(), A::add);\n";
const UNIT: &str = "laws::identity!(add_unit, any::<A>(), A::add, A::zero());\n";
const ZERO: &str = "laws::absorbing!(add_zero, any::<A>(), A::add, A::zero());\n";
const SPREAD: &str = "laws::distributive!(add_spreads, any::<A>(), A::add, A::add);\n";
const PLAIN_LAW: &str = "laws::law!(add_holds, any::<A>(), value, { A::add(value) });\n";
const TITLED: &str = "laws::associative!(A_add, any::<u8>(), other);\n";
const TYPE_ONLY: &str = "laws::associative!(regroups, any::<A>(), other);\n";
const FUNCTION_ONLY: &str = "laws::associative!(regroups, any::<u8>(), add);\n";
const NESTED: &str =
    "#[cfg(test)]\nmod tests {\n    laws::inverse!(cancels, any::<A>(), A::add, A::neg, A::zero());\n}\n";
const IN_FILE: &str = "struct A;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A {}\n}\n#[cfg(test)]\nmod tests {\n    laws::associative!(regroups, any::<A>(), A::add);\n}\n";
const FOREIGN_MACRO: &str = "other::associative!(regroups, any::<A>(), A::add);\n";
const BARE_MACRO: &str = "associative!(regroups, any::<A>(), A::add);\n";
const OTHER_ADD: &str = "struct B;\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B {}\n}\n";

#[test]
fn an_unlawed_add_is_listed_with_its_type_and_the_fix() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", ADD)]);
    assert!(
        text.contains("crates/x/src/lib.rs:4: crate x: 1 of 1 algebra operations declare no law; x::A: add; declare the law with laws::associative!, commutative!, identity!, inverse!, idempotent!, absorbing! or distributive! in a test of the same package, calling the operation as Type::function, <Type as Trait>::function or a closure that calls it on its parameters"),
        "{text}"
    );
}

#[test]
fn every_operation_and_partial_is_listed_by_type() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", MANY)]);
    assert!(
        text.contains("crate x: 5 of 5 algebra operations declare no law; x::A: zero, scale, get, bump; x::B: join"),
        "{text}"
    );
}

#[test]
fn counts_are_per_crate() {
    let text = Declared::of(&[
        Declared::at("crates/x/src/lib.rs", ADD),
        Declared::at("crates/y/src/lib.rs", OTHER_ADD),
        Declared::at("crates/y/tests/t.rs", "laws::associative!(regroups, any::<B>(), B::add);\n"),
    ]);
    assert!(text.contains("crate x: 1 of 1 algebra operations declare no law; x::A: add"), "{text}");
    assert_eq!(text.matches("crate y").count(), 0, "{text}");
}

#[test]
fn a_crate_counts_its_lawed_operations_in_the_total() {
    let text = Declared::of(&[
        Declared::at("crates/x/src/lib.rs", ADD),
        Declared::at("crates/x/src/other.rs", "struct B;\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n"),
        Declared::at("crates/x/tests/t.rs", CLEARING),
    ]);
    assert!(text.contains("crate x: 1 of 2 algebra operations declare no law; x::B: join"), "{text}");
}

#[test]
fn a_law_naming_the_type_and_the_function_clears_the_operation() {
    assert!(
        [CLEARING, COMMUTING, IDEMPOTENT, NESTED, UNIT, ZERO, SPREAD]
            .iter()
            .all(|law| Declared::lawed(ADD, law).matches("declare no law").count() == 0)
    );
}

#[test]
fn a_law_in_a_test_module_of_the_file_clears_the_operation() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", IN_FILE)]);
    assert_eq!(text.matches("declare no law").count(), 0, "{text}");
}

#[test]
fn a_law_of_another_package_does_not_attach() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", ADD), Declared::at("crates/y/tests/t.rs", CLEARING)]);
    assert!(text.contains("crate x: 1 of 1 algebra operations declare no law; x::A: add"), "{text}");
}

#[test]
fn a_label_a_plain_law_or_half_the_names_does_not_attach() {
    assert!(
        [TITLED, PLAIN_LAW, TYPE_ONLY, FUNCTION_ONLY, FOREIGN_MACRO]
            .iter()
            .all(|law| Declared::lawed(ADD, law).contains("x::A: add"))
    );
}

#[test]
fn a_bare_macro_name_attaches() {
    let text = Declared::lawed(ADD, BARE_MACRO);
    assert_eq!(text.matches("declare no law").count(), 0, "{text}");
}

#[test]
fn a_signature_a_foreign_trait_fixes_is_skipped() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", CLONED)]);
    assert_eq!(text.matches("declare no law").count(), 0, "{text}");
}

#[test]
fn std_operators_over_one_type_are_not_skipped() {
    let text = Declared::of(&[Declared::at("crates/x/src/lib.rs", MULTIPLIED)]);
    assert!(text.contains("crate x: 1 of 1 algebra operations declare no law; x::A: neg"), "{text}");
}

#[test]
fn transitions_and_test_only_code_are_not_algebra() {
    assert!(
        [TRANSITION, TESTED].iter().all(|source| Declared::of(&[Declared::at("crates/x/src/lib.rs", source)])
            .matches("declare no law")
            .count()
            == 0)
    );
}

#[test]
fn visibility_does_not_exempt_an_operation() {
    assert!(
        [PRIVATE, PUBLIC]
            .iter()
            .all(|source| Declared::of(&[Declared::at("crates/x/src/lib.rs", source)]).contains("x::A: join"))
    );
}

#[test]
fn a_preserves_law_is_not_a_law_of_the_operation() {
    let text = Declared::lawed(ADD, "laws::preserves!(keeps, any::<A>(), A::add, B::add, A::into_b);\n");
    assert!(text.contains("x::A: add"), "{text}");
}

const ONE_LINE: &str = "struct A;\nimpl A {\n    fn join(self, other: A) -> A {} fn merge(self, other: A) -> A {}\n}\n";

#[test]
fn two_operations_on_one_line_keep_their_own_laws() {
    let text = Declared::lawed(ONE_LINE, "laws::associative!(join_regroups, any::<A>(), A::join);\n");
    assert!(text.contains("crate x: 1 of 2 algebra operations declare no law; x::A: merge"), "{text}");
    let both = Declared::lawed(
        ONE_LINE,
        "laws::associative!(join_regroups, any::<A>(), A::join);\nlaws::associative!(merge_regroups, any::<A>(), A::merge);\n",
    );
    assert_eq!(both.matches("declare no law").count(), 0, "{both}");
}

const TWO_JOINS: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n}\n";

#[test]
fn a_law_credits_only_the_operation_its_call_routes_to() {
    [
        ("laws::associative!(b_join, any::<A>(), B::join);\n", "x::A: join", "x::B: join"),
        ("laws::associative!(a_join, any::<A>(), A::join);\n", "x::B: join", "x::A: join"),
    ]
    .iter()
    .map(|(law, unlawed, lawed)| {
        let text = Declared::lawed(TWO_JOINS, law);
        let listed = format!("crate x: 1 of 2 algebra operations declare no law; {unlawed}");
        (text.matches(&listed).count(), text.matches(lawed).count())
    })
    .for_each(|counts| assert_eq!(counts, (1, 0)));
}

const BOTH_ADDS: &str = "struct A;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A {}\n}\nstruct B;\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B {}\n}\n";
const B_ADD_FORMS: [&str; 7] = [
    "laws::associative!(add_regroups, any::<A>(), B::add);\n",
    "laws::associative!(add_regroups, any::<B>(), Add::add);\n",
    "laws::associative!(add_regroups, any::<A>(), <B as Add>::add);\n",
    "impl B {\n    fn regroups() {\n        laws::associative!(add_regroups, any::<A>(), Self::add);\n    }\n}\n",
    "use x::B as Sum;\nlaws::associative!(add_regroups, any::<A>(), Sum::add);\n",
    "use std::ops::Add as Plus;\nlaws::associative!(add_regroups, any::<B>(), Plus::add);\n",
    "fn regroups() {\n    use x::B as Sum;\n    laws::associative!(add_regroups, any::<A>(), Sum::add);\n}\n",
];

#[test]
fn every_path_form_attaches_to_the_type_it_resolves_to() {
    let listed: Vec<String> = B_ADD_FORMS.iter().map(|law| Declared::lawed(BOTH_ADDS, law)).collect();
    assert!(
        listed.iter().all(|text| text.contains("crate x: 1 of 2 algebra operations declare no law; x::A: add")),
        "{listed:?}"
    );
}

#[test]
fn a_trait_path_a_foreign_trait_names_does_not_attach() {
    let text = Declared::lawed(BOTH_ADDS, "laws::associative!(add_regroups, any::<B>(), Other::add);\n");
    assert!(text.contains("crate x: 2 of 2 algebra operations declare no law"), "{text}");
}

#[test]
fn a_closure_attaches_to_the_method_it_calls_on_its_parameters() {
    [
        "laws::associative!(a_join, any::<B>(), |a: A, b: A| a.join(b));\n",
        "laws::associative!(a_join, any::<A>(), |a, b| a.join(b));\n",
        "laws::associative!(a_join, any::<B>(), |a: A, b: A| { A::join(a, b) });\n",
    ]
    .iter()
    .map(|law| Declared::lawed(TWO_JOINS, law))
    .for_each(|text| {
        assert!(text.contains("crate x: 1 of 2 algebra operations declare no law; x::B: join"), "{text}");
    });
}

#[test]
fn a_closure_the_lint_cannot_resolve_is_named_with_its_site() {
    let text = Declared::lawed(PRIVATE, "laws::associative!(a_join, any::<A>(), |a: A, b: A| combine(a, b));\n");
    assert!(
        text.contains("crates/x/tests/t.rs:1: law associative! checks a closure the lint cannot resolve to an operation; pass the operation path (Type::function), or a closure that calls it on its parameters"),
        "{text}"
    );
}

#[test]
fn an_unreadable_closure_is_reported_when_every_operation_has_a_law_and_a_preserves_closure_is_not() {
    let lawed = "laws::associative!(a_join, any::<A>(), A::join);\nlaws::commutative!(a_swap, any::<A>(), |a: A, b: A| combine(a, b));\nlaws::preserves!(keeps, any::<A>(), A::join, |a: B, b: B| combine(a, b), A::into_b);\n";
    let text = Declared::lawed(PRIVATE, lawed);
    assert_eq!(
        (text.matches("law commutative! checks a closure").count(), text.matches("law preserves!").count()),
        (1, 0),
        "{text}"
    );
}

const JOINS_BESIDE_A_TRAIT: [(&str, &str); 2] = [
    (
        "crates/x/src/lib.rs",
        "struct A;\nstruct Join;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl Join {\n    fn join(self, other: Join) -> Join {}\n}\n",
    ),
    ("crates/y/src/lib.rs", "trait Join {}\n"),
];

#[test]
fn a_trait_of_another_crate_does_not_capture_a_type_of_the_same_name() {
    let mut sources: Vec<Source> = JOINS_BESIDE_A_TRAIT.iter().map(|&(path, text)| Declared::at(path, text)).collect();
    sources.push(Declared::at("crates/x/tests/t.rs", "laws::associative!(join_regroups, any::<A>(), Join::join);\n"));
    let text = Declared::of(&sources);
    assert!(text.contains("crate x: 1 of 2 algebra operations declare no law; x::A: join"), "{text}");
}

#[test]
fn a_law_counts_only_when_its_path_attaches() {
    [
        "laws::associative!(a_join, any::<A>(), Self::join);\n",
        "laws::associative!(A_join, any::<A>(), join);\n",
        "laws::associative!(a_join, any::<A>(), |a: A, b: A| b.join(a).join(a));\n",
        "laws::law!(a_join, any::<A>(), a, { A::join(a, a) });\n",
    ]
    .iter()
    .map(|law| Declared::lawed(PRIVATE, law))
    .for_each(|text| assert!(text.contains("crate x: 1 of 1 algebra operations declare no law; x::A: join"), "{text}"));
}

const WRAPPED: &str = "struct A;\nstruct B;\ntrait Join {}\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn join(self, other: B) -> B {}\n    fn zero() -> B {}\n}\n";
const WRAPPED_FORMS: [&str; 12] = [
    "laws::identity!(b_unit, any::<A>(), &B::join, B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), (B::join), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: &B, b: &B| a.join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: (B), b: B| a.join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: B, b: B| (a.join(b)), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: B, b: B| (a).join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: B, b: B| (&a).join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<A>(), |a: B, b: B| (*a).join(b), B::zero());\n",
    "laws::identity!(b_unit, (any::<B>()), |a, b| a.join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<B>().prop_map(|b| b), |a, b| a.join(b), B::zero());\n",
    "laws::identity!(b_unit, any::<B>(), Join::join, B::zero());\n",
    "impl B {\n    fn unit() {\n        laws::identity!(b_unit, any::<A>(), |a: Self, b: Self| a.join(b), Self::zero());\n    }\n}\n",
];

#[test]
fn a_wrapped_operation_operand_or_parameter_attaches_to_the_type_it_resolves_to() {
    let listed: Vec<String> = WRAPPED_FORMS.iter().map(|law| Declared::lawed(WRAPPED, law)).collect();
    assert!(
        listed.iter().all(|text| text.contains("crate x: 1 of 3 algebra operations declare no law; x::A: join")),
        "{listed:?}"
    );
}

const ONE_JOIN: &str = PRIVATE;
const Y_TRAIT: Extra = Extra { path: "crates/y/src/lib.rs", text: "pub trait Join {}\n" };
const Y_STRUCT: Extra = Extra { path: "crates/y/src/lib.rs", text: "pub struct Join;\n" };
const W_EMPTY: Extra = Extra { path: "crates/w/src/lib.rs", text: "pub struct Other;\n" };
const Z_REEXPORT: Extra = Extra { path: "crates/z/src/lib.rs", text: "pub use y::Join;\n" };
const Z_RENAMED: Extra = Extra { path: "crates/z/src/lib.rs", text: "pub use y::Join as Merge;\n" };
const Z_GLOB: Extra = Extra { path: "crates/z/src/lib.rs", text: "pub use y::*;\n" };
const Z_PRIVATE: Extra = Extra { path: "crates/z/src/lib.rs", text: "use y::Join;\n" };
const V_TWO_HOP: Extra = Extra { path: "crates/v/src/lib.rs", text: "pub use z::Join;\n" };
const X_TRAIT: Extra = Extra { path: "crates/x/src/ops.rs", text: "pub trait Join {}\n" };
const X_IMPORTS_Y: Extra = Extra { path: "crates/x/src/imports.rs", text: "use y::Join;\n" };
const P_GLOB: Extra = Extra { path: "crates/p/src/lib.rs", text: "pub use q::*;\n" };
const Q_GLOB: Extra = Extra { path: "crates/q/src/lib.rs", text: "pub use p::*;\n" };
const X_IMPORTS_W: Extra = Extra { path: "crates/x/src/imports.rs", text: "use w::Join;\n" };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Credited,
    Listed,
}

impl Expect {
    const LISTED: &str = "crate x: 1 of 1 algebra operations declare no law; x::A: join";

    fn listings(self) -> usize {
        match self {
            Self::Credited => 0,
            Self::Listed => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Extra {
    path: &'static str,
    text: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Row {
    id: &'static str,
    extras: &'static [Extra],
    law: &'static str,
    expect: Expect,
}

impl Row {
    const fn new(id: &'static str, extras: &'static [Extra], law: &'static str, expect: Expect) -> Self {
        Self { id, extras, law, expect }
    }

    fn failure(&self) -> Option<String> {
        let sources: Vec<Source> = std::iter::once(Declared::at("crates/x/src/lib.rs", ONE_JOIN))
            .chain(self.extras.iter().map(|extra| Declared::at(extra.path, extra.text)))
            .chain(std::iter::once(Declared::at("crates/x/tests/t.rs", self.law)))
            .collect();
        let text = Declared::of(&sources);
        Some(text.matches(Expect::LISTED).count())
            .filter(|listings| *listings != self.expect.listings())
            .map(|_| format!("{} expected {:?}: {text}", self.id, self.expect))
    }
}

const TRAIT_PATH_FORMS: [Row; 38] = [
    Row::new("A1", &[Y_TRAIT], "use y::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new(
        "A2",
        &[Y_TRAIT],
        "use y::Join as Merge;\nlaws::associative!(r, any::<A>(), Merge::join);\n",
        Expect::Credited,
    ),
    Row::new(
        "A3",
        &[Y_TRAIT],
        "use y::{Join, Other};\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Credited,
    ),
    Row::new("A4", &[Y_TRAIT], "use y::*;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A5", &[Y_TRAIT], "use y::ops::*;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A6", &[Y_TRAIT], "laws::associative!(r, any::<A>(), y::Join::join);\n", Expect::Credited),
    Row::new("A7", &[Y_TRAIT], "use y::ops;\nlaws::associative!(r, any::<A>(), ops::Join::join);\n", Expect::Credited),
    Row::new(
        "A8",
        &[Y_TRAIT],
        "use y as why;\nlaws::associative!(r, any::<A>(), why::Join::join);\n",
        Expect::Credited,
    ),
    Row::new(
        "A9",
        &[Y_TRAIT],
        "fn regroups() {\n    use y::*;\n    laws::associative!(r, any::<A>(), Join::join);\n}\n",
        Expect::Credited,
    ),
    Row::new(
        "A10",
        &[Y_TRAIT],
        "use y::*;\nimpl A {\n    fn regroups() {\n        laws::associative!(r, any::<A>(), Join::join);\n    }\n}\n",
        Expect::Credited,
    ),
    Row::new(
        "A11",
        &[Y_TRAIT],
        "use y::*;\nmod inner {\n    laws::associative!(r, any::<A>(), Join::join);\n}\n",
        Expect::Credited,
    ),
    Row::new(
        "A12",
        &[Y_TRAIT, Z_REEXPORT],
        "use z::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Credited,
    ),
    Row::new("A13", &[Y_TRAIT, Z_REEXPORT], "laws::associative!(r, any::<A>(), z::Join::join);\n", Expect::Credited),
    Row::new(
        "A14",
        &[Y_TRAIT, Z_RENAMED],
        "use z::Merge;\nlaws::associative!(r, any::<A>(), Merge::join);\n",
        Expect::Credited,
    ),
    Row::new(
        "A15",
        &[Y_TRAIT, Z_GLOB],
        "use z::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Credited,
    ),
    Row::new(
        "A16",
        &[V_TWO_HOP, Z_REEXPORT, Y_TRAIT],
        "use v::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Credited,
    ),
    Row::new("A17", &[X_TRAIT], "use crate::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A18", &[X_TRAIT], "use self::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A19", &[X_TRAIT], "use super::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A20", &[X_TRAIT], "laws::associative!(r, any::<A>(), crate::Join::join);\n", Expect::Credited),
    Row::new("A21", &[X_TRAIT], "laws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A22", &[X_TRAIT], "use x::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new(
        "A23",
        &[X_TRAIT],
        "use Join as Merge;\nlaws::associative!(r, any::<A>(), Merge::join);\n",
        Expect::Credited,
    ),
    Row::new("A24", &[Y_TRAIT, X_IMPORTS_Y], "laws::associative!(r, any::<A>(), Join::join);\n", Expect::Credited),
    Row::new("A25", &[], "laws::associative!(r, any::<A>(), From::from);\n", Expect::Listed),
    Row::new(
        "A26",
        &[P_GLOB, Q_GLOB],
        "use p::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
    Row::new("N1", &[Y_TRAIT], "laws::associative!(r, any::<A>(), Join::join);\n", Expect::Listed),
    Row::new("N2", &[Y_STRUCT], "use y::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Listed),
    Row::new(
        "N3",
        &[Y_TRAIT, W_EMPTY],
        "use w::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
    Row::new("N4", &[Y_TRAIT, W_EMPTY], "laws::associative!(r, any::<A>(), w::Join::join);\n", Expect::Listed),
    Row::new("N5", &[Y_TRAIT, W_EMPTY], "use w::*;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Listed),
    Row::new(
        "N6",
        &[Y_TRAIT, W_EMPTY],
        "use y::*;\nuse w::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
    Row::new(
        "N7",
        &[Y_TRAIT, Z_PRIVATE],
        "use z::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
    Row::new("N8", &[Y_TRAIT], "use crate::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n", Expect::Listed),
    Row::new("N9", &[Y_TRAIT], "laws::associative!(r, any::<A>(), crate::Join::join);\n", Expect::Listed),
    Row::new(
        "N10",
        &[X_TRAIT, Y_STRUCT],
        "use y::Join;\nlaws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
    Row::new("N11", &[], "laws::associative!(r, any::<A>(), Other::join);\n", Expect::Listed),
    Row::new(
        "N12",
        &[Y_TRAIT, W_EMPTY, X_IMPORTS_W],
        "laws::associative!(r, any::<A>(), Join::join);\n",
        Expect::Listed,
    ),
];

#[test]
fn a_trait_path_resolves_through_imports_globs_qualified_paths_and_re_exports() {
    let failing: Vec<String> = TRAIT_PATH_FORMS.iter().filter_map(Row::failure).collect();
    assert_eq!(failing, Vec::<String>::new());
}

const ROUNDTRIP_CLOSURE: &str = "laws::roundtrip!(rt, any::<A>(), |a: &A| encode(a), |b| decode(b));\n";
const PLAIN_LAW_CLOSURE: &str = "laws::law!(holds, any::<A>(), |a: A| check(a));\n";
const SWAP_CLOSURE: &str = "laws::commutative!(s, any::<A>(), |a: A, b: A| combine(a, b));\n";

#[test]
fn an_unresolved_closure_of_every_kind_but_preserves_is_reported() {
    let reported: Vec<usize> = [("roundtrip", ROUNDTRIP_CLOSURE), ("law", PLAIN_LAW_CLOSURE)]
        .iter()
        .map(|&(kind, law)| {
            Declared::lawed(PRIVATE, law)
                .matches(&format!(
                    "crates/x/tests/t.rs:1: law {kind}! checks a closure the lint cannot resolve to an operation"
                ))
                .count()
        })
        .collect();
    assert_eq!(reported, [1, 1]);
}

#[test]
fn an_unresolved_closure_outside_a_package_with_operations_is_not_reported() {
    let in_transitions = Declared::lawed(TRANSITION, "laws::idempotent!(settles, any::<A>(), |a: A| normalise(a));\n");
    let in_other_package = Declared::of(&[
        Declared::at("crates/x/src/lib.rs", PRIVATE),
        Declared::at("crates/y/tests/t.rs", SWAP_CLOSURE),
    ]);
    let in_both = Declared::of(&[
        Declared::at("crates/x/src/lib.rs", PRIVATE),
        Declared::at("crates/x/tests/t.rs", SWAP_CLOSURE),
        Declared::at("crates/y/tests/t.rs", SWAP_CLOSURE),
    ]);
    assert_eq!(
        [&in_transitions, &in_other_package, &in_both].map(|text| text.matches("checks a closure").count()),
        [0, 0, 1],
        "{in_transitions}{in_other_package}{in_both}"
    );
}
