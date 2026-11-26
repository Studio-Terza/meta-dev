use serde_json::error::Category;
use text_lines::{KeptLines, Lines, LossyText, NotOneLine, OneLine};

struct Sample;

impl Sample {
    fn bracketed(text: &str) -> String {
        text.parse::<Lines>().into_iter().flatten().map(|line| ["<", line.as_str(), ">"].concat()).collect()
    }

    fn kept(text: &str) -> String {
        text.parse::<KeptLines>()
            .into_iter()
            .flatten()
            .flat_map(|line| ["<".to_owned(), line.escape_debug().to_string(), ">".to_owned()])
            .collect()
    }
}

#[test]
fn text_splits_at_each_line_break_without_keeping_it() {
    for (text, want) in [
        ("", ""),
        ("a", "<a>"),
        ("a\nb", "<a><b>"),
        ("a\nb\n", "<a><b>"),
        ("a\n\nb", "<a><><b>"),
        ("\n", "<>"),
        ("\n\n", "<><>"),
        ("\na", "<><a>"),
        ("é\nü", "<é><ü>"),
        ("a b\tc\nd", "<a b\tc><d>"),
        ("a\r\nb", "<a\r><b>"),
    ] {
        assert_eq!(Sample::bracketed(text), want, "{text:?}");
    }
}

#[test]
fn text_splits_after_each_line_break_keeping_it() {
    for (text, want) in [
        ("", ""),
        ("a", "<a>"),
        ("a\nb", r"<a\n><b>"),
        ("a\nb\n", r"<a\n><b\n>"),
        ("a\n\nb", r"<a\n><\n><b>"),
        ("\n", r"<\n>"),
        ("\n\n", r"<\n><\n>"),
        ("\na", r"<\n><a>"),
        ("é\nü", r"<é\n><ü>"),
        ("a\r\nb", r"<a\r\n><b>"),
    ] {
        assert_eq!(Sample::kept(text), want, "{text:?}");
    }
}

#[test]
fn bytes_that_are_not_utf8_become_the_replacement_character() {
    for (bytes, want) in [
        (&b""[..], ""),
        (b"abc", "abc"),
        ("é".as_bytes(), "é"),
        (b"a\xffb", "a\u{FFFD}b"),
        (b"\xff\xfe", "\u{FFFD}\u{FFFD}"),
        (b"ok\xc3", "ok\u{FFFD}"),
    ] {
        assert_eq!(LossyText::from(bytes).to_string(), want, "{bytes:?}");
    }
}

struct Read;

impl Read {
    fn line(text: &str) -> Result<String, NotOneLine> {
        OneLine::try_new(text.to_owned()).map(|line| line.to_string())
    }

    fn deserialized(json: &str) -> Result<String, Category> {
        serde_json::from_str::<OneLine>(json).map(|line| line.as_ref().to_owned()).map_err(|error| error.classify())
    }
}

#[test]
fn text_with_a_line_break_or_control_character_is_not_one_line() {
    [("", Ok(String::new())), ("a b", Ok("a b".to_owned())), ("é ü", Ok("é ü".to_owned()))]
        .into_iter()
        .chain(["a\nb", "a\rb", "a\tb", "a\u{7f}", "\u{0}"].map(|text| (text, Err(NotOneLine))))
        .for_each(|(text, want)| assert_eq!(Read::line(text), want, "{text:?}"));
}

#[test]
fn a_line_read_from_json_is_checked() {
    assert_eq!(Read::deserialized("\"a b\""), Ok("a b".to_owned()));
    assert_eq!(Read::deserialized("\"a\\nb\""), Err(Category::Data));
    assert_eq!(Read::deserialized("1"), Err(Category::Data));
}
