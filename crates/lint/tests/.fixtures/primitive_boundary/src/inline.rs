use std::fmt;

pub struct Shown;
impl fmt::Display for Shown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("shown")
    }
}
impl PartialEq<str> for Shown {
    fn eq(&self, other: &str) -> bool {
        other.is_empty()
    }
}
mod boundary {
    pub fn read(text: &str) -> usize {
        text.len()
    }
}
mod elsewhere {
    pub fn read(text: &str) -> usize {
        text.len()
    }
}
macro_rules! made {
    () => {
        fn made(flag: bool) {}
    };
}
made!();
