use std::io;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use invariants::{Ignoring, Unpoisoned, ignored, invariant, invariants};
use proptest::prelude::{any, prop_assert_eq, proptest};
use test_failure::Failed;

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

proptest! {
    #[test]
    fn a_condition_passes_when_it_holds_and_panics_when_it_does_not(held in any::<bool>()) {
        let checked = panic::catch_unwind(|| invariant!(held, Invariant::Positive));
        prop_assert_eq!(checked.is_ok(), held);
    }

    #[test]
    fn a_held_result_gives_its_value_or_panics(count in any::<u16>()) {
        let held = panic::catch_unwind(|| invariant!(holding u8::try_from(count), Invariant::Narrow));
        prop_assert_eq!(held.into_iter().next(), u8::try_from(count).into_iter().next());
    }

    #[test]
    fn an_ignored_error_keeps_the_value(count in any::<u16>()) {
        prop_assert_eq!(u8::try_from(count).ignoring(Ignored::Wide), u8::try_from(count).into_iter().next());
    }

    #[test]
    fn a_lock_gives_its_guard(kept in any::<u8>()) {
        prop_assert_eq!(*Mutex::new(kept).lock().unpoisoned(), kept);
    }
}

#[test]
fn each_broken_invariant_and_reason_says_its_text() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let (messages, width): (PathBuf, u16) = serde_json::from_slice(&std::fs::read(found)?)?;
    let lock = Mutex::new(width);
    let poisoning = panic::catch_unwind(|| {
        let _held = lock.lock().unpoisoned();
        invariant!(broken Invariant::Positive)
    });
    let poisoned = panic::catch_unwind(|| *lock.lock().unpoisoned());
    let panics = [
        panic::catch_unwind(|| invariant!(holding u8::try_from(width), Invariant::Narrow)).map(drop),
        panic::catch_unwind(|| invariant!(holding u8::try_from(width), Invariant::Narrow, detailed)).map(drop),
        poisoning.map(drop),
        poisoned.map(drop),
    ];
    let said: Vec<String> = panics
        .into_iter()
        .flat_map(|outcome| match outcome {
            Ok(()) => Vec::new(),
            Err(payload) => vec![payload],
        })
        .filter_map(|payload| payload.downcast::<String>().into_iter().next().map(|text| *text))
        .chain([Ignored::Wide.to_string()])
        .collect();
    assert_eq!(said, serde_json::from_slice::<Vec<String>>(&std::fs::read(messages)?)?);
    Ok(())
}
