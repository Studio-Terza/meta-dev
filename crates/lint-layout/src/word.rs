use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Word<'word>(&'word str);

#[macro_export]
macro_rules! words {
    ($($word:expr),* $(,)?) => {
        [$($crate::Word::new($word)),*]
    };
}
