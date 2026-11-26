#![deny(clippy::wildcard_enum_match_arm)]

use grammar_lstart::{Stamp, Unstamped};

#[test]
fn a_process_start_time_is_read_as_two_words_and_five_numbers() {
    [
        ("Fri Nov  6 01:16:00 2026    \n", Ok((["Fri", "Nov"], [6, 1, 16, 0, 2026]))),
        ("Wed Sep 30 22:11:39 2026", Ok((["Wed", "Sep"], [30, 22, 11, 39, 2026]))),
        ("Fri Nov  6 01:16 2026", Err(Unstamped::Unshaped("Fri Nov  6 01:16 2026".to_owned()))),
        ("Fri Nov  6 01:16:00", Err(Unstamped::Unshaped("Fri Nov  6 01:16:00".to_owned()))),
        ("Fri Nov  6 01:16:00 2026 2027", Err(Unstamped::Unshaped("Fri Nov  6 01:16:00 2026 2027".to_owned()))),
        ("Fri Nov  6 01:16 00:2026", Err(Unstamped::Unshaped("Fri Nov  6 01:16 00:2026".to_owned()))),
        ("Fri Nov  6 01:16:00 Sat", Err(Unstamped::Misplaced("Fri Nov  6 01:16:00 Sat".to_owned()))),
        ("Fri 11  6 01:16:00 2026", Err(Unstamped::Misplaced("Fri 11  6 01:16:00 2026".to_owned()))),
        ("fri Nov  6 01:16:00 2026", Err(Unstamped::Unshaped("fri Nov  6 01:16:00 2026".to_owned()))),
        ("Fri Nov  6 01:16:00 99999", Err(Unstamped::Unshaped("Fri Nov  6 01:16:00 99999".to_owned()))),
        ("", Err(Unstamped::Unshaped(String::new()))),
    ]
    .into_iter()
    .for_each(|(text, want)| {
        assert_eq!(Stamp::try_from(text).map(|stamp| (stamp.words(), stamp.numbers())), want, "{text}");
    });
}
