use std::convert::Infallible;
use std::ffi::OsStr;
use std::str::FromStr;

use newtype::Newtype;

use crate::document::text::Text;
use crate::word::Word;

#[derive(Debug)]
pub(super) struct Inline(Vec<String>);

impl Inline {
    const EVAL: Word = Word::new("eval");
    const INTERPRETERS: [&'static str; 16] = [
        "bash",
        "sh",
        "zsh",
        "dash",
        "ksh",
        "fish",
        "pwsh",
        "powershell",
        "cmd",
        "python",
        "python3",
        "node",
        "deno",
        "perl",
        "ruby",
        "php",
    ];
    const FLAGS: [&'static str; 13] =
        ["-c", "-e", "-ec", "-ce", "-xc", "-pe", "-ne", "-r", "-Command", "-command", "--command", "--eval", "/c"];

    pub(super) fn program(self) -> Option<Word> {
        let interpreted = self
            .0
            .iter()
            .zip(self.0.iter().skip(1))
            .filter_map(|(program, flag)| Text::new(program).path().file_name().map(|name| (name, flag)))
            .filter(|(name, _)| Self::INTERPRETERS.iter().any(|known| *name == OsStr::new(known)))
            .find_map(|(_, flag)| Self::FLAGS.iter().copied().find(|known| flag == known))
            .map(Word::new);
        self.0.iter().any(|word| word == Self::EVAL.into_inner()).then_some(Self::EVAL).or(interpreted)
    }
}

impl FromStr for Inline {
    type Err = Infallible;

    fn from_str(shown: &str) -> Result<Inline, Infallible> {
        let letters: Vec<char> = shown.chars().collect();
        Ok(Inline(
            letters
                .chunk_by(|first, second| first.is_whitespace() == second.is_whitespace())
                .filter(|run| run.iter().all(|letter| !letter.is_whitespace()))
                .map(|run| run.iter().collect())
                .collect(),
        ))
    }
}
