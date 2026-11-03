#[derive(Debug)]
pub struct Letters;

impl Letters {
    pub fn humps(chunk: &str) -> Vec<String> {
        let letters: Vec<char> = chunk.chars().collect();
        let cuts = (0..letters.len()).filter(|index| Self::cut(&letters, *index).is_some());
        let bounds: Vec<usize> = cuts.chain(Some(letters.len())).collect();
        bounds
            .iter()
            .zip(bounds.iter().skip(1))
            .filter_map(|(start, end)| letters.get(*start..*end))
            .map(|hump| hump.iter().collect())
            .collect()
    }

    fn cut(letters: &[char], index: usize) -> Option<&char> {
        let at = |place: usize| letters.get(place).copied();
        let previous = index.checked_sub(1).and_then(at);
        let next = index.checked_add(1).and_then(at);
        (previous.is_none()
            || (at(index).is_some_and(char::is_uppercase)
                && previous.is_some_and(|before| {
                    before.is_lowercase()
                        || before.is_ascii_digit()
                        || (before.is_uppercase() && next.is_some_and(char::is_lowercase))
                })))
        .then(|| letters.get(index))
        .flatten()
    }
}
