//! The one place free-form agent prompt text is scanned. Its only typed content is a lease token.
#![allow(clippy::disallowed_methods)]

use crate::ids::LeaseId;

pub(crate) struct Prompt<'text>(&'text str);

impl<'text> Prompt<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        Self(text)
    }

    pub(crate) fn lease(&self) -> Option<LeaseId> {
        self.0
            .split(|letter: char| !(letter.is_ascii_alphanumeric() || letter == '-'))
            .find_map(|word| word.parse::<LeaseId>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_lease_among_words() {
        let found = Prompt::new("You are x.\nlease-4, go").lease();
        assert_eq!(found, Some("lease-4".parse().unwrap()));
    }

    #[test]
    fn no_lease_is_none() {
        assert_eq!(Prompt::new("task-4 only").lease(), None);
    }
}
