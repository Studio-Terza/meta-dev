#![deny(clippy::wildcard_enum_match_arm)]

use grammar_search::Terms;

struct Lexed;

impl Lexed {
    fn terms(words: &[&str]) -> Vec<String> {
        Terms::lexed(words.iter().copied())
            .map_or_default(|terms| [terms.first()].into_iter().chain(terms.rest()).map(ToString::to_string).collect())
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
        ["かなカナ—naïve", "café…ok"] => vec!["かなカナ", "naïve", "café", "ok"],
        ["一、二。三"] => vec!["一", "二", "三"],
    ]
);
