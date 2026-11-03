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
        let first = (index == 0).then_some(());
        let after_lower = previous.filter(|before| before.is_lowercase());
        let after_digit = previous.filter(char::is_ascii_digit);
        let acronym_end =
            previous.filter(|before| before.is_uppercase()).and(next.filter(|after| after.is_lowercase()));
        let humped = at(index).filter(|letter| letter.is_uppercase()).and(after_lower.or(after_digit).or(acronym_end));
        first.or(humped.map(|_| ())).and(letters.get(index))
    }
}
