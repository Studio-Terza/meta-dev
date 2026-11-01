use crate::name::Name;
use crate::word::Word;

pub trait Matching {
    fn holds(name: &Name, word: Word<'_>) -> bool;
}
