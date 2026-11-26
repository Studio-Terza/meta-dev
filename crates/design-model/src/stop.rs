use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gist {
    Question,
    Statement,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Awaited {
    Owner,
    Shell,
    Agent,
    Nothing,
    Unclassified,
    Unchecked,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    Live,
    Deadlock,
    Overrun,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    Blocked,
    Passed,
}
