#![deny(clippy::wildcard_enum_match_arm)]

use grammar_line::Unbroken;

laws::table!(
    one_unbroken_line_with_a_visible_character_is_spelled_whole,
    |text: &&str| Unbroken::spelled(text).map(str::to_owned),
    [
        "a condition" => Some("a condition".to_owned()),
        "  padded " => Some("  padded ".to_owned()),
        "\tx" => Some("\tx".to_owned()),
        "" => None,
        "  " => None,
        "\t" => None,
        "a\nb" => None,
        "a\r\n" => None,
        "\na" => None,
        "\u{b}" => None,
        "\u{c}" => None,
        "\u{a0}" => None,
        "\u{2028}" => None,
        "\u{85}" => None,
        "\u{3000}\t" => None,
        "\u{a0}x" => Some("\u{a0}x".to_owned()),
        "x\u{b}y" => None,
        "x\u{c}y" => None,
        "x\u{85}y" => None,
        "x\u{2028}y" => None,
        "x\u{2029}y" => None,
        "x\ry" => None,
        "x\u{2029}" => None,
        "\u{85}x" => None,
        "x\u{2007}y" => Some("x\u{2007}y".to_owned()),
    ]
);
