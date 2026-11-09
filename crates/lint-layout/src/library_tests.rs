#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LibraryTests {
    #[default]
    Run,
    SwitchedOff,
}

impl From<bool> for LibraryTests {
    fn from(tests: bool) -> LibraryTests {
        tests.then_some(LibraryTests::Run).unwrap_or(LibraryTests::SwitchedOff)
    }
}
