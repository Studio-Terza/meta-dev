use std::panic;
use std::sync::Mutex;

use fixture::{Cases, Debugged, Failed, Fixture};
use invariants::{Ignoring, Unpoisoned, ignored, invariant, invariants};
use serde::Deserialize;

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

#[derive(Fixture, Deserialize, Debug)]
struct Outcomes;

#[test]
fn invariants_hold_quietly_and_break_loudly_with_their_text_and_detail() -> Result<(), Failed> {
    let drifted = Cases::<Outcomes>::drifted(|_case| {
        let said = |outcome: std::thread::Result<u8>| {
            outcome.map_err(|payload| payload.downcast::<String>().into_iter().map(|text| *text).collect::<String>())
        };
        let counts = [u16::from(u8::MAX), u16::MAX];
        let narrowed = counts.map(|count| {
            said(panic::catch_unwind(|| match u8::try_from(count) {
                Ok(narrow) => narrow,
                Err(wide) => invariant!(broken Invariant::Narrow, wide),
            }))
        });
        let held = counts
            .map(|count| said(panic::catch_unwind(|| invariant!(holding u8::try_from(count), Invariant::Narrow))));
        let detailed = counts.map(|count| {
            said(panic::catch_unwind(|| invariant!(holding u8::try_from(count), Invariant::Narrow, detailed)))
        });
        let checked = counts.map(|count| u8::try_from(count).is_ok()).map(|holds| {
            said(panic::catch_unwind(|| {
                invariant!(holds, Invariant::Positive);
                u8::MAX
            }))
        });
        let lock = Mutex::new(u8::MAX);
        let poisoning = said(panic::catch_unwind(|| {
            let _held = lock.lock().unpoisoned();
            invariant!(broken Invariant::Positive)
        }));
        let poisoned = said(panic::catch_unwind(|| *lock.lock().unpoisoned()));
        let ignored = counts.map(|count| u8::try_from(count).ignoring(Ignored::Wide));
        let reasons = [Ignored::Wide.to_string(), Invariant::Positive.to_string()];
        Ok(Debugged((narrowed, held, detailed, checked, poisoning, poisoned, ignored, reasons)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
