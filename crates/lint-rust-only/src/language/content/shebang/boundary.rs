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
            .filter(|chunk| chunk.iter().all(|letter| !letter.is_whitespace()))
            .map(|chunk| chunk.iter().collect())
            .collect()
    }
}

impl FromStr for Shebang {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<Shebang, Infallible> {
        let words = Self::words(line);
        let program = words.first().map(String::as_str).and_then(Self::named);
        let interpreter = match program {
            Some(Self::LAUNCHER) => words
                .iter()
                .skip(1)
                .find(|word| !matches!(word.chars().next(), Some('-')) && !word.contains('='))
                .map(String::as_str)
                .and_then(Self::named),
            other => other,
        };
        Ok(Shebang(interpreter.map(str::to_owned)))
    }
}
