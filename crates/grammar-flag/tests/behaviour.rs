#![deny(clippy::wildcard_enum_match_arm)]

use grammar_flag::Flags;

laws::table!(
    a_word_led_by_two_dashes_is_spelled_as_a_flag,
    |text: &&str| Flags::spelled(text).map(str::to_owned),
    [
        "--reason" => Some("--reason".to_owned()),
        "--" => Some("--".to_owned()),
        "---x" => Some("---x".to_owned()),
        "--no such" => Some("--no such".to_owned()),
        "-r" => None,
        "demo" => None,
        "a--b" => None,
        " --reason" => None,
        "" => None,
    ]
);
