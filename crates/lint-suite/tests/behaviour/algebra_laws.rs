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
        text.contains("crates/x/src/lib.rs:4: crate x: 1 of 1 algebra operations declare no law; x::A: add; declare the law with laws::associative!, commutative!, identity!, inverse!, idempotent!, absorbing! or distributive! in a test of the same package, naming the type and the function"),
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
