use lint_layout::Whole;
use lint_layout::{Name, Word, words};

#[derive(Debug)]
pub(crate) struct FeatureWord;

impl FeatureWord {
    const TEST_WORDS: [Word<'static>; 5] = words!["test", "tests", "testing", "mock", "mocks"];

    pub(crate) fn test_word(feature: &Name) -> Option<Word<'static>> {
        feature.words().iter().find_map(|word| word.matches::<Whole>(&Self::TEST_WORDS))
    }
}
