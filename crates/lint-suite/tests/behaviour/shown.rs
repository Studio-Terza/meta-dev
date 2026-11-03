use lint_core::{Lint, SourceFile};
use lint_fixtures::{Sources, Text};

pub(crate) struct Shown;

impl Shown {
    pub(crate) fn by(pair: &[&str], reported: impl Fn(&[SourceFile]) -> String) -> bool {
        Some(pair).filter(|pair| pair.len() == 2).and_then(|pair| pair.first().zip(pair.last())).is_some_and(
            |(text, expected)| reported(&[Sources::file("crates/x/src/lib.rs", Text::new(text))]).contains(expected),
        )
    }

    pub(crate) fn of<Checked: Lint>(pair: &[&str]) -> bool {
        Self::by(pair, |files| Checked::report(files).to_string())
    }
}
