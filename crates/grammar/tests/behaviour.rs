use grammar::ident::Words;
use grammar::lstart::Stamp;
use grammar::message::{Depth, Lexemes, Mark, Number};
use grammar::node::{Ids, Spelled};
use grammar::provenance::Agents;
use grammar::search::Terms;
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

struct Lexed;

impl Lexed {
    fn terms(words: &[&str]) -> Vec<String> {
        Terms::lexed(words.iter().copied())
            .map(|terms| [terms.first()].into_iter().chain(terms.rest()).map(ToString::to_string).collect())
            .unwrap_or_default()
    }
}

laws::table!(
    search_words_lex_into_lowered_terms_split_at_every_other_character,
    Lexed::terms,
    [
        ["Pinning-tests", "NOW"] => vec!["pinning", "tests", "now"],
        ["\"a OR b*\"", "c:d"] => vec!["a", "or", "b", "c", "d"],
        ["Écoute", "v0.2"] => vec!["écoute", "v0", "2"],
        ["--", "..."] => Vec::<String>::new(),
    ]
);

laws::table!(
    node_ids_are_n_and_a_number_without_leading_zeros_that_fits,
    Ids::spelled,
    [
        "n0" => Spelled::Numbered(0),
        "n42" => Spelled::Numbered(42),
        "n9223372036854775807" => Spelled::Numbered(i64::MAX),
        "n00" => Spelled::Misnumbered,
        "n001" => Spelled::Misnumbered,
        "n9223372036854775808" => Spelled::Misnumbered,
        "n99999999999999999999" => Spelled::Misnumbered,
        "n" => Spelled::Unspelled,
        "42" => Spelled::Unspelled,
        "n42x" => Spelled::Unspelled,
    ]
);

laws::table!(
    a_process_start_time_is_read_as_two_words_and_five_numbers,
    |text: &&'static str| Stamp::read(text).map(|stamp| (stamp.words(), stamp.numbers())),
    [
        "Fri Nov  6 01:16:00 2026    \n" => Some((["Fri", "Nov"], [6, 1, 16, 0, 2026])),
        "Wed Sep 30 22:11:39 2026" => Some((["Wed", "Sep"], [30, 22, 11, 39, 2026])),
        "Fri Nov  6 01:16 2026" => None,
        "Fri Nov  6 01:16:00" => None,
        "Fri Nov  6 01:16:00 2026 2027" => None,
        "fri Nov  6 01:16:00 2026" => None,
        "Fri Nov  6 01:16:00 99999" => None,
        "" => None,
    ]
);

laws::table!(
    an_agent_provenance_is_read_as_its_run_number,
    |text: &&str| Agents::numbered(text),
    [
        "agent 7" => Some(7),
        "agent -3" => Some(-3),
        "agent 0" => Some(0),
        "agent" => None,
        "agent x" => None,
        "agent 7 " => None,
        "apply" => None,
        "agent 99999999999999999999" => None,
    ]
);
