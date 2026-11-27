use super::NoTestOnlyApi;

impl NoTestOnlyApi<'_> {
    pub(super) fn testing(feature: &str) -> Option<()> {
        let letters: Vec<char> = feature.chars().collect();
        letters
            .chunk_by(|left, right| left.is_alphanumeric() == right.is_alphanumeric())
            .any(|word| {
                let spelled: String = word.iter().collect();
                NoTestOnlyApi::TEST_WORDS.contains(&spelled.as_str())
            })
            .then_some(())
    }
}
