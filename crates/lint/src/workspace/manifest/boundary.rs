use std::path::Component;

use super::Members;

impl Members {
    const ANY: char = '*';

    pub(super) fn fitting(expected: &Component<'_>, part: &Component<'_>) -> Option<()> {
        let letters: Vec<char> = expected.as_os_str().to_string_lossy().chars().collect();
        let spelled: Vec<char> = part.as_os_str().to_string_lossy().chars().collect();
        Members::fits(&letters, &spelled)
    }

    fn fits(pattern: &[char], text: &[char]) -> Option<()> {
        match pattern.split_first() {
            None => text.is_empty().then_some(()),
            Some((&Members::ANY, rest)) => Members::fits(rest, text).or_else(|| Members::fits(pattern, text.get(1..)?)),
            Some((letter, rest)) => {
                let (first, tail) = text.split_first()?;
                (first == letter).then_some(()).and_then(|()| Members::fits(rest, tail))
            }
        }
    }
}
