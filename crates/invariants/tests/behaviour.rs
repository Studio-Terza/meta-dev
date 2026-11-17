use std::fmt;
use std::panic;

use invariants::invariant;

#[derive(Clone, Copy, Debug)]
enum Invariant {
    Positive,
}

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
        outcome.err().and_then(|payload| payload.downcast::<String>().ok()).map(|text| *text)
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
    let crashed = panic::catch_unwind(|| {
        u8::try_from(300_u16).map_or_else(|_wide| invariant!(broken Invariant::Positive), |_narrow| ());
    });
    assert_eq!(Caught::message(crashed).as_deref(), Some("broken invariant: the count is positive"));
}
