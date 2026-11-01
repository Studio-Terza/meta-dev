mod cuts;
mod rule;
mod scan_bytes;
mod shebang;
mod stride;
mod weight;

use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use newtype::Newtype;

use super::Language;
use cuts::Cuts;
use rule::Rule;
use scan_bytes::ScanBytes;
use shebang::Shebang;
use stride::Stride;
use weight::Weight;

#[derive(Debug)]
pub(crate) struct Content {
    text: String,
}

impl Content {
    const WINDOW: ScanBytes = ScanBytes::new(8192);
    const FENCE: &'static str = "```";
    const NEWLINE: &'static str = "\n";
    const RETURN: char = '\r';
    const EVERY_OTHER: Stride = Stride::new(2);

    pub(crate) fn language_of(path: &Path) -> Option<Language> {
        Shebang::read_from(path)
            .filter(Shebang::is_foreign)
            .map(|_| Language::Script)
            .or_else(|| Self::read_from(path)?.language())
    }

    fn read_from(path: &Path) -> Option<Content> {
        Self::WINDOW
            .bytes(path)
            .filter(|bytes| !bytes.contains(&b'\0'))
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .or_else(|failure| {
                        let valid = failure.utf8_error().valid_up_to();
                        let mut kept = failure.into_bytes();
                        kept.truncate(valid);
                        String::from_utf8(kept)
                    })
                    .ok()
            })
            .and_then(|text| text.parse().ok())
    }

    fn language(&self) -> Option<Language> {
        Rule::ALL.iter().map(Rule::language).find(|language| self.evidence_for(*language) >= Rule::ENOUGH)
    }

    fn evidence_for(&self, language: Language) -> Weight {
        Weight::new(
            self.unfenced_lines()
                .flat_map(|line| {
                    Rule::ALL
                        .iter()
                        .filter(move |rule| rule.language() == language)
                        .filter_map(move |rule| rule.weighs(line))
                })
                .map(Weight::into_inner)
                .fold(u8::MIN, u8::saturating_add),
        )
    }

    fn unfenced_lines(&self) -> impl Iterator<Item = &str> {
        Self::EVERY_OTHER
            .every_nth(Cuts::new(&self.text, Self::FENCE).pieces())
            .flat_map(|outside| Cuts::new(outside, Self::NEWLINE).pieces())
    }
}

impl FromStr for Content {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Content, Infallible> {
        Ok(Content { text: text.chars().filter(|letter| *letter != Self::RETURN).collect() })
    }
}
