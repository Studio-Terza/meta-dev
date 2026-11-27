mod lexeme;
mod reading;

use reading::Reading;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flagged {
    Rm,
    Rmdir,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Construct {
    ShellScript,
    Loop,
    Conditional,
    Function,
    Subshell,
    Group,
    Substitution,
    Xargs,
    Find,
    Heredoc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Occurrence {
    flagged: Flagged,
    within: Vec<Construct>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nested {
    flagged: Flagged,
    construct: Construct,
}

impl Nested {
    #[must_use]
    pub fn read(command: &str) -> Option<Nested> {
        Reading::occurrences(command).iter().find_map(Occurrence::nested)
    }

    #[must_use]
    pub fn flagged(self) -> Flagged {
        self.flagged
    }

    #[must_use]
    pub fn construct(self) -> Construct {
        self.construct
    }
}

impl Occurrence {
    fn nested(&self) -> Option<Nested> {
        self.within.first().map(|construct| Nested { flagged: self.flagged, construct: *construct })
    }

    fn inside(self, outer: &[Construct]) -> Occurrence {
        Occurrence { within: outer.iter().copied().chain(self.within).collect(), flagged: self.flagged }
    }
}
