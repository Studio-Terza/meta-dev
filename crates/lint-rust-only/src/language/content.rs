mod cuts;
mod rule;
mod scan_bytes;
mod shebang;
mod stride;
mod text;
mod weight;

use std::convert::Infallible;
use std::str::FromStr;

use file_ports::Bytes;
use lint_core::Contents;
use newtype::Newtype;

use super::Language;
use cuts::Cuts;
use rule::Rule;
use scan_bytes::ScanBytes;
use shebang::Shebang;
use stride::Stride;
use text::Text;
use weight::Weight;

#[derive(Debug)]
pub(crate) struct Content(String);

impl Content {
    const WINDOW: ScanBytes = ScanBytes::new(8192);
    const FENCE: Text<'static> = Text::new("```");
    const NEWLINE: Text<'static> = Text::new("\n");
    const RETURN: char = '\r';
    const EVERY_OTHER: Stride = Stride::new(2);

    pub(crate) fn language_of(contents: &Contents) -> Option<Language> {
        let bytes = contents.read().ok()?;
        Shebang::read_from(bytes)
            .and_then(Shebang::foreign)
            .map(|_| Language::Script)
            .or_else(|| Self::read_from(bytes)?.language())
    }

    fn read_from(contents: &Bytes) -> Option<Content> {
        Some(Self::WINDOW.bytes(contents))
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

    fn unfenced_lines(&self) -> impl Iterator<Item = Text<'_>> {
        Self::EVERY_OTHER
            .every_nth(Cuts::new(Text::new(&self.0), Self::FENCE).pieces())
            .flat_map(|outside| Cuts::new(outside, Self::NEWLINE).pieces())
    }
}

impl FromStr for Content {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Content, Infallible> {
        Ok(Content(text.chars().filter(|letter| *letter != Self::RETURN).collect()))
    }
}
