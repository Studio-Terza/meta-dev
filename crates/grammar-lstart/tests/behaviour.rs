#![deny(clippy::wildcard_enum_match_arm)]

use grammar_lstart::Stamp;

laws::table!(
    a_process_start_time_is_read_as_two_words_and_five_numbers,
    |text: &&'static str| Stamp::read(text).map(|stamp| (stamp.words(), stamp.numbers())),
    [
        "Fri Nov  6 01:16:00 2026    \n" => Some((["Fri", "Nov"], [6, 1, 16, 0, 2026])),
        "Wed Sep 30 22:11:39 2026" => Some((["Wed", "Sep"], [30, 22, 11, 39, 2026])),
        "Fri Nov  6 01:16 2026" => None,
        "Fri Nov  6 01:16:00" => None,
        "Fri Nov  6 01:16:00 2026 2027" => None,
        "Fri Nov  6 01:16 00:2026" => None,
        "Fri Nov  6 01:16:00 Sat" => None,
        "Fri 11  6 01:16:00 2026" => None,
        "fri Nov  6 01:16:00 2026" => None,
        "Fri Nov  6 01:16:00 99999" => None,
        "" => None,
    ]
);
