#![deny(clippy::wildcard_enum_match_arm)]

use grammar_message::{Depth, Lexeme, Lexemes, Mark, Number};
use proptest::prelude::{prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};

struct Marks;

impl Marks {
    fn of(text: &str) -> Vec<Mark> {
        Lexemes::scanned(&text).into_iter().map(|lexeme| lexeme.mark()).filter(|mark| *mark != Mark::Plain).collect()
    }
}

#[test]
fn markers_are_recognised_at_line_starts() {
    let top = |number| Mark::Numbered(Number::new(number), Depth::Top);
    assert_eq!(Marks::of("1. one\n2) two"), vec![top(1), top(2)]);
    assert_eq!(Marks::of("1. one\n  2. sub"), vec![top(1), Mark::Numbered(Number::new(2), Depth::Nested)]);
    assert_eq!(Marks::of("**1. one**\n**2.** two"), vec![top(1), top(2)]);
    assert_eq!(Marks::of("- a\n* b\n> c"), vec![Mark::Bulleted, Mark::Bulleted, Mark::Quoted]);
    assert_eq!(Marks::of("yes. no? ok"), vec![Mark::Stop, Mark::Stop]);
    assert_eq!(Marks::of("v0.2.1 is fine"), Vec::<Mark>::new());
    assert_eq!(Marks::of("see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b now"), vec![Mark::Uuid]);
    assert_eq!(Marks::of("line\nnext"), vec![Mark::Break]);
}

struct Inks;

impl Inks {
    fn of(text: &str) -> String {
        Lexemes::scanned(&text).iter().filter_map(Lexeme::ink).map(|ink| ink.slice(text)).collect()
    }
}

#[test]
fn review_a_lexeme_ink_sheds_unicode_whitespace() {
    assert_eq!(Inks::of("agreed.\u{a0}"), "agreed.");
    assert_eq!(Inks::of("\u{3000}fine"), "fine");
    assert_eq!(Inks::of("\u{2003}one\u{85}two\u{2029}"), "onetwo");
    assert_eq!(Inks::of("\u{200b}kept"), "\u{200b}kept");
}

struct Inked;

impl Inked {
    fn expected(mark: Mark) -> Option<bool> {
        match mark {
            Mark::Break => Some(false),
            Mark::Stop | Mark::Numbered(..) | Mark::Bulleted | Mark::Quoted | Mark::Uuid => Some(true),
            Mark::Plain => None,
        }
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn lexemes_tile_the_text_without_gaps(text in "[ -~\n\t•é]{0,60}") {
        let rebuilt: String = Lexemes::scanned(&text).iter().map(|lexeme| lexeme.span().slice(&text)).collect();
        prop_assert_eq!(rebuilt, text);
    }

    #[test]
    fn only_breaks_lack_ink_among_marks(
        text in "(\n- |\n> |\n1\\. |\n  2\\) |\n\\*\\*3\\.\\*\\* |[.?!] |85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b|[ a-z]|\n){0,24}"
    ) {
        let unexpected: Vec<Mark> = Lexemes::scanned(&text)
            .into_iter()
            .filter(|lexeme| Inked::expected(lexeme.mark()).is_some_and(|inked| inked != lexeme.ink().is_some()))
            .map(|lexeme| lexeme.mark())
            .collect();
        prop_assert_eq!(unexpected, Vec::<Mark>::new());
    }
}
