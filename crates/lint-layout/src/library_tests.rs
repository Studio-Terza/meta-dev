#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LibraryTests {
    #[default]
    Run,
    SwitchedOff,
}
