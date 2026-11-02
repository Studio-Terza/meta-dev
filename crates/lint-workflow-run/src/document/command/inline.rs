use std::convert::Infallible;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub(super) struct Inline(Vec<String>);

impl Inline {
    const EVAL: &'static str = "eval";
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

    pub(super) fn program(&self) -> Option<&'static str> {
        self.0.iter().any(|word| word == Self::EVAL).then_some(Self::EVAL).or_else(|| Self::flagged(&self.0))
    }

    fn flagged(words: &[String]) -> Option<&'static str> {
        words
            .iter()
            .zip(words.iter().skip(1))
            .filter_map(|(program, flag)| Self::interpreter(program).map(|_| flag))
            .find_map(|flag| Self::FLAGS.iter().copied().find(|known| flag == known))
    }

    fn interpreter(program: &str) -> Option<&str> {
        Path::new(program)
            .file_name()
            .filter(|name| Self::INTERPRETERS.iter().any(|known| *name == OsStr::new(known)))
            .map(|_| program)
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
