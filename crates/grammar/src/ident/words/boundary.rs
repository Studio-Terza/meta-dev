use logos::Logos;

use super::Words;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Hump {
    #[regex(r"[A-Z][^A-Z]*")]
    Capitalized,
    #[regex(r"[^A-Z]+")]
    Leading,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Snake {
    #[regex(r"[^_]+")]
    Word,
    #[token("_")]
    Underscore,
}

impl Words {
    #[must_use]
    pub fn lowered_humps(text: &str) -> Vec<String> {
        Hump::lexer(text)
            .spanned()
            .filter_map(|(_, range)| text.get(range))
            .map(|word| word.chars().flat_map(char::to_lowercase).collect())
            .collect()
    }

    #[must_use]
    pub fn camel(text: &str) -> String {
        Snake::lexer(text)
            .spanned()
            .flat_map(|(piece, range)| Snake::raised(piece, text.get(range).unwrap_or_default()))
            .collect()
    }
}

impl Snake {
    fn raised(piece: Result<Snake, ()>, written: &str) -> Vec<char> {
        match piece {
            Ok(Snake::Word) => Snake::capitalized(written),
            Ok(Snake::Underscore) | Err(()) => Vec::new(),
        }
    }

    fn capitalized(word: &str) -> Vec<char> {
        word.chars().take(1).flat_map(char::to_uppercase).chain(word.chars().skip(1)).collect()
    }
}
