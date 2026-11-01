use text_lines::{Lines, LossyText, OneLine};

struct Sample;

impl Sample {
    fn bracketed(text: &&str) -> String {
        Lines::from(*text).map(|line| ["<", line, ">"].concat()).collect()
    }
}

laws::table!(
    text_splits_at_each_line_break_without_keeping_it,
    Sample::bracketed,
    [
        "" => "",
        "a" => "<a>",
        "a\nb" => "<a><b>",
        "a\nb\n" => "<a><b>",
        "a\n\nb" => "<a><><b>",
        "\n" => "<>",
        "\n\n" => "<><>",
        "\na" => "<><a>",
        "é\nü" => "<é><ü>",
        "a b\tc\nd" => "<a b\tc><d>",
        "a\r\nb" => "<a\r><b>",
    ]
);

struct Decoded;

impl Decoded {
    fn text(bytes: &&[u8]) -> String {
        LossyText::from(*bytes).to_string()
    }
}

laws::table!(
    bytes_that_are_not_utf8_become_the_replacement_character,
    Decoded::text,
    [
        b"" as &[u8] => "",
        b"abc" as &[u8] => "abc",
        "é".as_bytes() => "é",
        b"a\xffb" as &[u8] => "a\u{FFFD}b",
        b"\xff\xfe" as &[u8] => "\u{FFFD}\u{FFFD}",
        b"ok\xc3" as &[u8] => "ok\u{FFFD}",
    ]
);

struct Read;

impl Read {
    fn line(text: &&str) -> Result<String, ()> {
        OneLine::try_from((*text).to_owned()).map(|line| line.to_string()).map_err(|_| ())
    }

    fn deserialized(json: &&str) -> Result<String, ()> {
        serde_json::from_str::<OneLine>(json).map(|line| line.as_ref().to_owned()).map_err(|_| ())
    }
}

laws::table!(
    text_with_a_line_break_or_control_character_is_not_one_line,
    Read::line,
    [
        "" => Ok(String::new()),
        "a b" => Ok("a b".to_owned()),
        "é ü" => Ok("é ü".to_owned()),
        "a\nb" => Err(()),
        "a\rb" => Err(()),
        "a\tb" => Err(()),
        "a\u{7f}" => Err(()),
        "\u{0}" => Err(()),
    ]
);

laws::table!(
    a_line_read_from_json_is_checked,
    Read::deserialized,
    [
        "\"a b\"" => Ok("a b".to_owned()),
        "\"a\\nb\"" => Err(()),
        "1" => Err(()),
    ]
);
