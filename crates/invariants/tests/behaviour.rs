use std::fmt;
use std::panic;

use invariants::{Ignore, Ignoring, invariant};

#[derive(Clone, Copy, Debug)]
enum Invariant {
    Positive,
}

#[derive(Clone, Copy, Debug)]
enum Ignored {
    Wide,
}

impl fmt::Display for Ignored {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ignored::Wide => formatter.write_str("a count too wide for a byte is not a byte count"),
        }
    }
}

impl Ignore for Ignored {}

impl fmt::Display for Invariant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invariant::Positive => formatter.write_str("the count is positive"),
        }
    }
}

struct Caught;

impl Caught {
    fn message(outcome: std::thread::Result<()>) -> Option<String> {
        match outcome {
            Ok(()) => None,
            Err(payload) => payload.downcast::<String>().into_iter().next().map(|text| *text),
        }
    }
}

#[test]
fn a_holding_invariant_passes_and_a_broken_one_panics_naming_it() {
    let holding = panic::catch_unwind(|| invariant!(2_u8 > 1, Invariant::Positive));
    assert_eq!(Caught::message(holding), None);
    let broken = panic::catch_unwind(|| invariant!(1_u8 > 2, Invariant::Positive));
    assert_eq!(Caught::message(broken).as_deref(), Some("broken invariant: the count is positive"));
}

#[test]
fn an_error_arm_crashes_on_a_named_invariant() {
    let crashed = panic::catch_unwind(|| invariant!(holding u8::try_from(300_u16), Invariant::Positive));
    assert_eq!(Caught::message(crashed.map(|_| ())).as_deref(), Some("broken invariant: the count is positive"));
    let held = panic::catch_unwind(|| invariant!(holding u8::try_from(30_u16), Invariant::Positive));
    assert_eq!(held.into_iter().next(), Some(30));
}

#[test]
fn an_ignored_error_names_its_reason_and_keeps_the_value() {
    assert_eq!(u8::try_from(300_u16).ignoring(Ignored::Wide), None);
    assert_eq!(u8::try_from(30_u16).ignoring(Ignored::Wide), Some(30));
}
