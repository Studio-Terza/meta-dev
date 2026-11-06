use grammar::ident::Words;
use grammar::message::{Depth, Lexemes, Mark, Number};
use grammar::slug::Letters;
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

laws::table!(
    slugs_keep_letters_and_digits_and_replace_every_other_character,
    Letters::slugged,
    [
        "/Users/me/my_repo" => "-Users-me-my-repo",
        "abc123" => "abc123",
        "a.b é" => "a-b--",
        "" => "",
    ]
);

laws::table!(
    camel_names_split_into_lowered_humps,
    Words::lowered_humps,
    [
        "Transcripts" => vec!["transcripts"],
        "ReadFrom" => vec!["read", "from"],
        "lowerThenUpper" => vec!["lower", "then", "upper"],
        "" => Vec::<String>::new(),
    ]
);

laws::table!(
    snake_names_raise_into_camel_case,
    Words::camel,
    [
        "read_from" => "ReadFrom",
        "sessions" => "Sessions",
        "a__b_" => "AB",
        "" => "",
    ]
);
