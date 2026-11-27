#[derive(Debug)]
pub(super) struct Glob;

#[derive(Debug)]
pub(super) struct Pattern(pub String);

impl Glob {
    const ANY: char = '*';

    pub(super) fn fits(pattern: &[char], text: &[char]) -> Option<()> {
        match pattern.split_first() {
            None => text.is_empty().then_some(()),
            Some((&Glob::ANY, rest)) => Glob::fits(rest, text).or_else(|| Glob::fits(pattern, text.get(1..)?)),
            Some((letter, rest)) => {
                let (first, tail) = text.split_first()?;
                (first == letter).then_some(()).and_then(|()| Glob::fits(rest, tail))
            }
        }
    }
}
