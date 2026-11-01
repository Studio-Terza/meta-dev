use lint_layout::Matching;
use lint_layout::Name;

#[derive(Debug)]
pub struct Suffix;

impl Matching for Suffix {
    fn holds(name: &Name, word: &str) -> bool {
        name.words().last().is_some_and(|last| last.lowered() == word)
    }
}
