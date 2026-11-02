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
        let profile = after.get(..split)?;
        let tail = after.get(split + directory.len()..)?;
        let (close, _) = tail.match_indices(Self::CLOSE).next()?;
        let name = tail.get(..close)?;
        (!name.is_empty()
            && !name.contains(char::is_whitespace)
            && !profile.is_empty()
            && !profile.contains(Self::SLASH))
        .then(|| Binary(name.to_owned()))
    }

    fn doctests(line: &str) -> Option<String> {
        let name: String = Self::after_indent(line, Self::DOCTESTS)?
            .chars()
            .skip_while(|letter| letter.is_whitespace())
            .take_while(|letter| !letter.is_whitespace())
            .collect();
        (!name.is_empty()).then(|| format!("{}{name}", Self::DOCTEST_PREFIX))
    }
}

impl FromStr for Binary {
    type Err = Unmatched;

    fn from_str(line: &str) -> Result<Binary, Unmatched> {
        Self::running(line).or_else(|| Self::doctests(line).map(Binary)).ok_or(Unmatched)
    }
}
