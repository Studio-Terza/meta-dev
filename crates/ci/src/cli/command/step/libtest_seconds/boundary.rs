use std::str::FromStr;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finished(Seconds);

#[derive(Debug, PartialEq, Eq)]
pub struct NotAResult;

impl Finished {
    const RESULT: &'static str = "test result";
    const MARK: &'static str = "in";
    const DECIMAL_POINT: char = '.';

    pub fn seconds(self) -> Seconds {
        self.0
    }

    fn decimal(letter: char) -> Option<char> {
        Some(letter)
            .filter(char::is_ascii_digit)
            .or_else(|| Some(letter).filter(|letter| *letter == Self::DECIMAL_POINT))
    }

    fn duration_after(words: &[String]) -> Seconds {
        words
            .iter()
            .zip(words.iter().skip(1))
            .filter(|(mark, _)| *mark == Self::MARK)
            .filter_map(|(_, seconds)| {
                seconds.chars().map_while(Self::decimal).collect::<String>().parse().ok().map(Seconds::new)
            })
            .sum()
    }
}

impl FromStr for Finished {
    type Err = NotAResult;

    fn from_str(line: &str) -> Result<Finished, NotAResult> {
        let letters: Vec<char> = line.chars().collect();
        let words: Vec<String> = letters
            .chunk_by(|left, right| left.is_whitespace() == right.is_whitespace())
            .filter_map(|chunk| chunk.first().filter(|letter| letter.is_whitespace()).map_or(Some(chunk), |_| None))
            .map(|chunk| chunk.iter().collect())
            .collect();
        (line.get(..Self::RESULT.len()) == Some(Self::RESULT))
            .then(|| Finished(Self::duration_after(&words)))
            .ok_or(NotAResult)
    }
}
