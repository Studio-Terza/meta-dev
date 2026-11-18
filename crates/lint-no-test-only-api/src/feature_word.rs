use lint_layout::{Fit, Matching, Name, Whole, Word, words};

#[derive(Debug)]
pub(crate) struct FeatureWord;

impl FeatureWord {
    pub(crate) const TEST_WORDS: [Word<'static>; 5] = words!["test", "tests", "testing", "mock", "mocks"];
}

impl Matching for FeatureWord {
    fn holds(feature: &Name, word: Word<'_>) -> Fit {
        Fit::from(feature.words().iter().find(|chunk| Whole::holds(chunk, word) == Fit::Fits))
    }
}
