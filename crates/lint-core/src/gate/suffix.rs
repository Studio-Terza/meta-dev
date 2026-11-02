use lint_layout::Matching;
use lint_layout::Name;
use lint_layout::Word;

#[derive(Debug)]
pub struct Suffix;

impl Matching for Suffix {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.words().last().filter(|last| last.lowered() == word).map(|_| word)
    }
}
