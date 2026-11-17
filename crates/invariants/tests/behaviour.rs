use std::panic;

use invariants::{invariant, invariants};

invariants! {
    enum Invariant {
        Positive => "the count is positive",
        Narrow => "the count fits a byte",
    }
}

struct Caught;

impl Caught {
    fn narrowed(count: u16) -> u8 {
        match u8::try_from(count) {
            Ok(narrow) => narrow,
            Err(wide) => invariant!(broken Invariant::Narrow, wide),
        }
    }

    fn message<Kept>(outcome: std::thread::Result<Kept>) -> Option<String> {
        match outcome {
            Ok(_) => None,
            Err(payload) => payload.downcast::<String>().into_iter().next().map(|text| *text),
        }
    }
}

#[test]
fn each_declared_invariant_states_its_own_text() {
    assert_eq!(Invariant::Positive.to_string(), "the count is positive");
    assert_eq!(Invariant::Narrow.to_string(), "the count fits a byte");
}

#[test]
fn a_holding_invariant_passes_and_a_broken_one_panics_with_its_text() {
    let holding = panic::catch_unwind(|| invariant!(2_u8 > 1, Invariant::Positive));
    assert_eq!(Caught::message(holding), None);
    let broken = panic::catch_unwind(|| invariant!(1_u8 > 2, Invariant::Positive));
    assert_eq!(Caught::message(broken).as_deref(), Some("broken invariant: the count is positive"));
}

#[test]
fn an_error_arm_crashes_on_a_named_invariant_and_its_detail() {
    let crashed = panic::catch_unwind(|| Caught::narrowed(300));
    let said = Caught::message(crashed);
    assert_eq!(
        said.as_deref(),
        Some("broken invariant: the count fits a byte: number too large to fit in target type")
    );
}
