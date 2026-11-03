use std::str::FromStr;

use super::unmatched::Unmatched;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binary(String);

impl Binary {
    const RUNNING: &'static str = "Running ";
    const DEPS: &'static str = "(target/";
    const DEPS_DIRECTORY: &'static str = "/deps/";
    const DOCTESTS: &'static str = "Doc-tests ";
    const DOCTEST_PREFIX: &'static str = "doc-";
    const CLOSE: char = ')';
    const SLASH: char = '/';

    pub(crate) fn label(&self) -> &str {
        &self.0
    }

    fn after_indent<'line>(line: &'line str, marker: &str) -> Option<&'line str> {
        line.match_indices(marker)
            .next()
            .filter(|(at, _)| line.get(..*at).is_some_and(|before| before.chars().all(char::is_whitespace)))
            .and_then(|(at, mark)| line.get(at + mark.len()..))
    }

    fn running(line: &str) -> Option<Binary> {
        let rest = Self::after_indent(line, Self::RUNNING)?;
        let (at, mark) = rest.match_indices(Self::DEPS).last()?;
        let after = rest.get(at + mark.len()..)?;
        let (split, directory) = after.match_indices(Self::DEPS_DIRECTORY).next()?;
        let tail = after
            .get(..split)
            .and_then(|profile| Self::word(profile, |letter| letter == Self::SLASH))
            .and(after.get(split + directory.len()..))?;
        let (close, _) = tail.match_indices(Self::CLOSE).next()?;
        let name = tail.get(..close).and_then(|name| Self::word(name, char::is_whitespace))?;
        Some(Binary(name.to_owned()))
    }

    fn word(text: &str, mut separator: impl FnMut(char) -> bool) -> Option<&str> {
        let end = text.char_indices().find(|(_, letter)| separator(*letter)).map_or(text.len(), |(at, _)| at);
        text.get(..end).filter(|word| word.len() == text.len()).filter(|_| end > 0)
    }

    fn doctests(line: &str) -> Option<String> {
        let rest = Self::after_indent(line, Self::DOCTESTS)?;
        let leading: usize = rest.chars().take_while(|letter| letter.is_whitespace()).map(char::len_utf8).sum();
        let name = rest.get(leading..)?;
        let end = name.char_indices().find(|(_, letter)| letter.is_whitespace()).map_or(name.len(), |(at, _)| at);
        name.get(..end).filter(|_| end > 0).map(|name| format!("{}{name}", Self::DOCTEST_PREFIX))
    }
}

impl FromStr for Binary {
    type Err = Unmatched;

    fn from_str(line: &str) -> Result<Binary, Unmatched> {
        Self::running(line).or_else(|| Self::doctests(line).map(Binary)).ok_or(Unmatched)
    }
}
