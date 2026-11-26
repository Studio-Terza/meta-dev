use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    Live,
    Stalled,
    Overrun,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    Blocked,
    Passed,
}
