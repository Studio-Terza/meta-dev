//! A failed command's stderr, as the one line of text it is shown as: no surrounding blanks and
//! no trailing newline.
#![allow(clippy::disallowed_methods)]

use std::fmt;

pub(super) struct Stderr(String);

impl Stderr {
    pub(super) fn of(bytes: &[u8]) -> Self {
        Self(String::from_utf8_lossy(bytes).trim().to_owned())
    }
}

impl fmt::Display for Stderr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Stderr;

    #[test]
    fn stderr_shows_without_surrounding_blanks_or_the_trailing_newline() {
        assert_eq!(Stderr::of(b"  boom\n").to_string(), "boom");
        assert_eq!(Stderr::of(b"two\nlines\n\n").to_string(), "two\nlines");
        assert_eq!(Stderr::of(b"").to_string(), "");
    }
}
