use lint_layout::{Name, Whole};

#[derive(Debug)]
pub(crate) struct FeatureWord;

impl FeatureWord {
    const TEST_WORDS: [&str; 5] = ["test", "tests", "testing", "mock", "mocks"];

    pub(crate) fn is_test(feature: &Name) -> bool {
        feature.words().iter().any(|word| word.matches::<Whole>(&Self::TEST_WORDS))
    }
}
