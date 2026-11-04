use lint_core::Lint;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::AlgebraDuplicates;

use crate::shown::OnGraph;

struct Twinned;

impl Twinned {
    fn of(sources: &[Source]) -> String {
        OnGraph::<AlgebraDuplicates>::report(&Sources::parsed(sources)).to_string()
    }

    fn lawed(source: &'static str, laws: &'static str) -> String {
        Self::of(&[Self::at("crates/x/src/lib.rs", source), Self::at("crates/x/tests/t.rs", laws)])
    }

    fn at(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }
}

const JOINS: &str =
    "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn merge(self, other: A) -> A {}\n}\n";
const JOIN_LAWS: &str = "laws::associative!(join_regroups, any::<A>(), A::join);\nlaws::associative!(merge_regroups, any::<A>(), A::merge);\n";
const ADD_SUB: &str =
    "struct A;\nimpl A {\n    fn add(self, other: A) -> A {}\n    fn sub(self, other: A) -> A {}\n}\n";
const ADD_SUB_LAWS: &str =
    "laws::commutative!(add_reorders, any::<A>(), A::add);\nlaws::associative!(sub_regroups, any::<A>(), A::sub);\n";
const PARAMETERS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn scale(self, factor: B) -> A {}\n    fn shift(self, factor: C) -> A {}\n}\n";
const PARAMETER_LAWS: &str = "laws::associative!(scale_regroups, any::<A>(), A::scale);\nlaws::associative!(shift_regroups, any::<A>(), A::shift);\n";
const PARTIAL: &str =
    "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn try_join(self, other: A) -> Option<A> {}\n}\n";
const PARTIAL_LAWS: &str = "laws::associative!(join_regroups, any::<A>(), A::join);\nlaws::associative!(try_join_regroups, any::<A>(), A::try_join);\n";
const PARTIALS: &str = "struct A;\nimpl A {\n    fn try_join(self, other: A) -> Option<A> {}\n    fn try_merge(self, other: A) -> Result<A, E> {}\n}\n";
const PARTIALS_LAWS: &str = "laws::associative!(try_join_regroups, any::<A>(), A::try_join);\nlaws::associative!(try_merge_regroups, any::<A>(), A::try_merge);\n";
const THREE: &str = "struct A;\nimpl A {\n    fn join(self, other: A) -> A {}\n    fn merge(self, other: A) -> A {}\n    fn fuse(self, other: A) -> A {}\n}\n";
const THREE_LAWS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::associative!(b, any::<A>(), A::merge);\nlaws::associative!(c, any::<A>(), A::fuse);\n";
const BOTH_LAWS: &str = "laws::associative!(a, any::<A>(), A::join);\nlaws::commutative!(b, any::<A>(), A::join);\nlaws::commutative!(c, any::<A>(), A::merge);\nlaws::associative!(d, any::<A>(), A::merge);\n";
const BARE: &str = "struct A;\nimpl A {\n    fn zero() -> A {}\n    fn one() -> A {}\n}\n";
const BARE_LAWS: &str = "laws::identity!(a, any::<A>(), A::zero);\nlaws::identity!(b, any::<A>(), A::one);\n";
const TWO_TYPES: &str = "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A) -> A {}\n}\nimpl B {\n    fn merge(self, other: B) -> B {}\n}\n";
const TWO_TYPES_LAWS: &str =
    "laws::associative!(a, any::<A>(), A::join);\nlaws::associative!(b, any::<B>(), B::merge);\n";
const ELSEWHERE: &str = "struct B;\nimpl B {\n    fn merge(self, other: B) -> B {}\n}\n";

#[test]
fn two_lawed_operations_with_one_signature_and_law_are_reported() {
    let text = Twinned::lawed(JOINS, JOIN_LAWS);
    assert!(
        text.contains("crates/x/src/lib.rs:3: crate x: 2 operations in 1 group share a signature and laws; x::A (x::A, x::A) -> x::A: join, merge (Associative); keep one operation of each group, or give the others a different signature or a law of their own"),
        "{text}"
    );
}

#[test]
fn operations_with_different_laws_are_not_twins() {
    let text = Twinned::lawed(ADD_SUB, ADD_SUB_LAWS);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn operations_with_different_parameter_types_are_not_twins() {
    let text = Twinned::lawed(PARAMETERS, PARAMETER_LAWS);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn a_partial_and_a_total_operation_are_not_twins() {
    let text = Twinned::lawed(PARTIAL, PARTIAL_LAWS);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn two_partial_operations_are_twins() {
    let text = Twinned::lawed(PARTIALS, PARTIALS_LAWS);
    assert!(text.contains("x::A (x::A, x::A) -> partial x::A: try_join, try_merge (Associative)"), "{text}");
}

#[test]
fn a_lone_lawed_operation_is_not_a_group() {
    let text = Twinned::lawed(JOINS, "laws::associative!(join_regroups, any::<A>(), A::join);\n");
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn unlawed_operations_are_not_compared() {
    let text = Twinned::of(&[Twinned::at("crates/x/src/lib.rs", JOINS)]);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn the_group_lists_every_member_and_counts_them() {
    let text = Twinned::lawed(THREE, THREE_LAWS);
    assert!(text.contains("3 operations in 1 group"), "{text}");
    assert!(text.contains("join, merge, fuse (Associative)"), "{text}");
}

#[test]
fn laws_are_compared_as_a_set() {
    let text = Twinned::lawed(JOINS, BOTH_LAWS);
    assert!(text.contains("x::A (x::A, x::A) -> x::A: join, merge (Associative, Commutative)"), "{text}");
}

#[test]
fn groups_are_counted_per_crate_and_per_type() {
    let text = Twinned::of(&[
        Twinned::at("crates/x/src/lib.rs", JOINS),
        Twinned::at("crates/x/tests/t.rs", JOIN_LAWS),
        Twinned::at("crates/y/src/lib.rs", ELSEWHERE),
        Twinned::at("crates/y/tests/t.rs", "laws::associative!(a, any::<B>(), B::merge);\n"),
    ]);
    assert!(text.contains("crate x: 2 operations in 1 group"), "{text}");
    assert_eq!(text.matches("crate y").count(), 0, "{text}");
}

#[test]
fn operations_of_different_types_are_not_twins() {
    let text = Twinned::lawed(TWO_TYPES, TWO_TYPES_LAWS);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn constants_with_one_law_are_twins() {
    let text = Twinned::lawed(BARE, BARE_LAWS);
    assert!(text.contains("() -> x::A: zero, one (Identity)"), "{text}");
}

#[test]
fn several_groups_of_one_crate_are_counted_together() {
    let text = Twinned::of(&[
        Twinned::at("crates/x/src/lib.rs", JOINS),
        Twinned::at(
            "crates/x/src/other.rs",
            "struct B;\nimpl B {\n    fn fuse(self, other: B) -> B {}\n    fn blend(self, other: B) -> B {}\n}\n",
        ),
        Twinned::at(
            "crates/x/tests/t.rs",
            "laws::associative!(a, any::<A>(), A::join);\nlaws::associative!(b, any::<A>(), A::merge);\nlaws::commutative!(c, any::<B>(), B::fuse);\nlaws::commutative!(d, any::<B>(), B::blend);\n",
        ),
    ]);
    assert!(
        text.contains("crates/x/src/lib.rs:3: crate x: 4 operations in 2 groups share a signature and laws; x::A "),
        "{text}"
    );
    assert!(text.contains("; x::B (x::B, x::B) -> x::B: fuse, blend (Commutative)"), "{text}");
}

const ABSORB: &str =
    "struct A;\nimpl A {\n    fn absorb(&mut self, other: A) {}\n    fn join(self, other: A) -> A {}\n}\n";
const ABSORB_LAWS: &str =
    "laws::associative!(a, any::<A>(), A::absorb);\nlaws::associative!(b, any::<A>(), A::join);\n";
const ABSORBS: &str =
    "struct A;\nimpl A {\n    fn absorb(&mut self, other: A) {}\n    fn fold(&mut self, other: A) {}\n}\n";
const ABSORBS_LAWS: &str =
    "laws::associative!(a, any::<A>(), A::absorb);\nlaws::associative!(b, any::<A>(), A::fold);\n";

#[test]
fn an_in_place_and_a_returning_operation_are_not_twins() {
    let text = Twinned::lawed(ABSORB, ABSORB_LAWS);
    assert_eq!(text.matches("share a signature").count(), 0, "{text}");
}

#[test]
fn two_in_place_operations_are_twins() {
    let text = Twinned::lawed(ABSORBS, ABSORBS_LAWS);
    assert!(text.contains("x::A (x::A, x::A) -> x::A: absorb, fold (Associative)"), "{text}");
}

#[test]
fn a_crate_with_one_group_says_group() {
    let text = Twinned::lawed(JOINS, JOIN_LAWS);
    assert!(text.contains("in 1 group share"), "{text}");
    assert_eq!(text.matches("1 groups").count(), 0, "{text}");
}
