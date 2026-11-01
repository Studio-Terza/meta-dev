use crate::name::Name;

pub trait Matching {
    fn holds(name: &Name, word: &str) -> bool;
}
