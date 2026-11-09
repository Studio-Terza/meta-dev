#![deny(clippy::wildcard_enum_match_arm)]

use grammar_message::{Depth, Lexeme, Lexemes, Mark, Number};
use laws::proptest::prelude::prop_assert_eq;

struct Marks;

impl Marks {
    fn of(text: &str) -> Vec<Mark> {
        Lexemes::scanned(&text).into_iter().map(|lexeme| lexeme.mark()).filter(|mark| *mark != Mark::Plain).collect()
    }
}

laws::table!(
    markers_are_recognised_at_line_starts,
    Marks::of,
    [
        "1. one\n2) two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "1. one\n  2. sub" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Nested)],
        "**1. one**\n**2.** two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "- a\n* b\n> c" => vec![Mark::Bulleted, Mark::Bulleted, Mark::Quoted],
        "yes. no? ok" => vec![Mark::Stop, Mark::Stop],
        "v0.2.1 is fine" => Vec::<Mark>::new(),
        "see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b now" => vec![Mark::Uuid],
        "line\nnext" => vec![Mark::Break],
    ]
);

laws::law!(lexemes_tile_the_text_without_gaps, "[ -~\n\t•é]{0,60}", text, {
    let rebuilt: String = Lexemes::scanned(&text).iter().map(|lexeme| lexeme.span().slice(&text)).collect();
    prop_assert_eq!(rebuilt, text);
});

struct Inks;

impl Inks {
    fn of(text: &&str) -> String {
        Lexemes::scanned(text).iter().filter_map(Lexeme::ink).map(|ink| ink.slice(text)).collect()
    }
}

laws::table!(
    review_a_lexeme_ink_sheds_unicode_whitespace,
    Inks::of,
    [
        "agreed.\u{a0}" => "agreed.".to_owned(),
        "\u{3000}fine" => "fine".to_owned(),
        "\u{2003}one\u{85}two\u{2029}" => "onetwo".to_owned(),
        "\u{200b}kept" => "\u{200b}kept".to_owned(),
    ]
);
