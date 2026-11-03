use std::fmt;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Word<'word>(&'word str);

impl fmt::Display for Word<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[macro_export]
macro_rules! words {
    ($($word:expr),* $(,)?) => {
        [$($crate::Word::new($word)),*]
    };
}
