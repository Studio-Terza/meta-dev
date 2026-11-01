mod scan_bytes;

use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use scan_bytes::ScanBytes;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Shebang {
    interpreter: Option<String>,
}

impl Shebang {
    const MARK: &'static str = "#!";
    const SCANNED: ScanBytes = ScanBytes::new(256);
    const LAUNCHER: &'static str = "env";
    const RUST: [&'static str; 2] = ["rust-script", "cargo"];

    pub(crate) fn read_from(path: &Path) -> Option<Shebang> {
        let line = Self::SCANNED.first_line(path)?;
        line.get(..Self::MARK.len())
            .filter(|start| *start == Self::MARK)
            .and(line.get(Self::MARK.len()..))?
            .parse()
            .ok()
    }

    pub(crate) fn is_foreign(&self) -> bool {
        self.interpreter.as_deref().is_none_or(|name| !Self::RUST.contains(&name))
    }

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

    fn is_argument(word: &str) -> bool {
        matches!(word.chars().next(), Some('-')) || word.contains('=')
    }
}

impl FromStr for Shebang {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<Shebang, Infallible> {
        let words = Self::words(line);
        let program = words.first().map(String::as_str).and_then(Self::named);
        let interpreter = match program {
            Some(Self::LAUNCHER) => {
                words.iter().skip(1).find(|word| !Self::is_argument(word)).map(String::as_str).and_then(Self::named)
            }
            other => other,
        };
        Ok(Shebang { interpreter: interpreter.map(str::to_owned) })
    }
}
