use std::panic;

use invariants::{Ignoring, ignored, invariant, invariants};

invariants! {
    enum Invariant {
        Positive => "the count is positive",
        Narrow => "the count fits a byte",
    }
}

ignored! {
    enum Ignored {
        Wide => "a count too wide for a byte is not a byte count",
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
fn each_declared_invariant_and_reason_states_its_own_text() {
    assert_eq!(Invariant::Positive.to_string(), "the count is positive");
    assert_eq!(Invariant::Narrow.to_string(), "the count fits a byte");
    assert_eq!(Ignored::Wide.to_string(), "a count too wide for a byte is not a byte count");
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
    assert_eq!(
        Caught::message(crashed).as_deref(),
        Some("broken invariant: the count fits a byte: number too large to fit in target type")
    );
}

#[test]
fn a_held_result_gives_its_value_or_crashes_on_the_named_invariant() {
    let crashed = panic::catch_unwind(|| invariant!(holding u8::try_from(300_u16), Invariant::Narrow));
    assert_eq!(Caught::message(crashed).as_deref(), Some("broken invariant: the count fits a byte"));
    let held = panic::catch_unwind(|| invariant!(holding u8::try_from(30_u16), Invariant::Narrow));
    assert_eq!(held.into_iter().next(), Some(30));
}

#[test]
fn an_ignored_error_names_its_reason_and_keeps_the_value() {
    assert_eq!(u8::try_from(300_u16).ignoring(Ignored::Wide), None);
    assert_eq!(u8::try_from(30_u16).ignoring(Ignored::Wide), Some(30));
}
