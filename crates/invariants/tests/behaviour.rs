use std::panic;
use std::sync::Mutex;

use invariants::{Ignoring, Rejoined, Unpoisoned, ignored, invariant, invariants};

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

const POISONED: &str =
    "no thread panics while it holds a lock, so no lock is poisoned; were one, its state may be half-written";

struct Caught;

impl Caught {
    fn narrowed(count: u16) -> u8 {
        match u8::try_from(count) {
            Ok(narrow) => narrow,
            Err(wide) => invariant!(broken Invariant::Narrow, wide),
        }
    }

    fn said(outcome: std::thread::Result<u8>) -> String {
        match outcome {
            Ok(value) => format!("gave {value}"),
            Err(payload) => payload.downcast::<String>().into_iter().map(|text| *text).collect(),
        }
    }
}

#[test]
fn invariants_hold_quietly_and_break_loudly_with_their_text_and_detail() {
    let lock = Mutex::new(7_u8);
    let cases: [(fn() -> u8, String); 4] = [
        (|| Caught::narrowed(30), "gave 30".to_owned()),
        (
            || Caught::narrowed(300),
            "broken invariant: the count fits a byte: number too large to fit in target type".to_owned(),
        ),
        (
            || invariant!(holding u8::try_from(300_u16), Invariant::Narrow),
            "broken invariant: the count fits a byte".to_owned(),
        ),
        (
            || invariant!(holding u8::try_from(300_u16), Invariant::Narrow, detailed),
            "broken invariant: the count fits a byte: number too large to fit in target type".to_owned(),
        ),
    ];
    let said: Vec<String> = cases.iter().map(|(run, _)| Caught::said(panic::catch_unwind(run))).collect();
    assert_eq!(said, cases.map(|(_, want)| want));
    let checked = [2_u8 > 1, 1_u8 > 2].map(|holds| {
        Caught::said(panic::catch_unwind(|| {
            invariant!(holds, Invariant::Positive);
            1
        }))
    });
    assert_eq!(checked, ["gave 1", "broken invariant: the count is positive"]);
    let held = Caught::said(panic::catch_unwind(|| invariant!(holding u8::try_from(30_u16), Invariant::Narrow)));
    let poisoning = Caught::said(panic::catch_unwind(|| {
        let _held = lock.lock().unpoisoned();
        invariant!(broken Invariant::Positive)
    }));
    let poisoned = Caught::said(panic::catch_unwind(|| *lock.lock().unpoisoned()));
    let joined = Caught::said(panic::catch_unwind(|| std::thread::spawn(|| 5_u8).join().rejoined()));
    let carried = Caught::said(panic::catch_unwind(|| {
        std::thread::spawn(|| invariant!(broken Invariant::Narrow)).join().rejoined()
    }));
    assert_eq!(
        [held, poisoning, poisoned, joined, carried],
        [
            "gave 30".to_owned(),
            "broken invariant: the count is positive".to_owned(),
            format!("broken invariant: {POISONED}"),
            "gave 5".to_owned(),
            "broken invariant: the count fits a byte".to_owned()
        ]
    );
    assert_eq!(
        [u8::try_from(300_u16).ignoring(Ignored::Wide), u8::try_from(30_u16).ignoring(Ignored::Wide)],
        [None, Some(30)]
    );
    assert_eq!(Ignored::Wide.to_string(), "a count too wide for a byte is not a byte count");
}
