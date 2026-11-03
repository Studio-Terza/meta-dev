use lint_layout::{Matching, Name, Whole, Word, words};

#[derive(Debug)]
pub(crate) struct FeatureWord;

impl FeatureWord {
    pub(crate) const TEST_WORDS: [Word<'static>; 5] = words!["test", "tests", "testing", "mock", "mocks"];
}

impl Matching for FeatureWord {
    fn holds<'word>(feature: &Name, word: Word<'word>) -> Option<Word<'word>> {
        feature.words().iter().find_map(|chunk| Whole::holds(chunk, word))
    }
}
