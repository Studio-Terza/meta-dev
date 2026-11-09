use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use super::Shebang;

impl Shebang {
    const LAUNCHER: &'static str = "env";

    fn named(path: &str) -> Option<&str> {
        Path::new(path).file_name().and_then(|name| name.to_str())
    }

    fn words(line: &str) -> Vec<String> {
        line.chars()
            .collect::<Vec<char>>()
            .chunk_by(|first, second| first.is_whitespace() == second.is_whitespace())
            .filter(|chunk| Run::from(*chunk) == Run::Word)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    fn interpreter(words: &[String]) -> Option<&str> {
        words
            .iter()
            .skip(1)
            .find(|word| Argument::from(word.as_str()) == Argument::Program)
            .map(String::as_str)
            .and_then(Self::named)
    }
}

impl FromStr for Shebang {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<Shebang, Infallible> {
        let words = Self::words(line);
        let program = words.first().map(String::as_str).and_then(Self::named);
        let interpreter =
            program.filter(|candidate| *candidate == Self::LAUNCHER).map_or(program, |_| Self::interpreter(&words));
        Ok(Shebang(interpreter.map(str::to_owned)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Run {
    Space,
    Word,
}

impl From<&[char]> for Run {
    fn from(chunk: &[char]) -> Run {
        chunk.first().filter(|letter| letter.is_whitespace()).map_or(Run::Word, |_| Run::Space)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Argument {
    Flag,
    Assignment,
    Program,
}

impl From<&str> for Argument {
    fn from(word: &str) -> Argument {
        word.chars()
            .next()
            .filter(|first| *first == '-')
            .map(|_| Argument::Flag)
            .or_else(|| word.chars().find(|letter| *letter == '=').map(|_| Argument::Assignment))
            .unwrap_or(Argument::Program)
    }
}
