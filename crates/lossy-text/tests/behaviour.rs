use lossy_text::LossyText;

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
