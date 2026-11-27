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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    Sed,
    Perl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    InPlace(Editor),
    Wrote(String),
    Moved { sources: Vec<String>, destination: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Spotted {
    Flagged(Flagged),
    Edit(Edit),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Occurrence {
    spotted: Spotted,
    within: Vec<Construct>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nested {
    flagged: Flagged,
    construct: Construct,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    edit: Edit,
    construct: Option<Construct>,
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

impl Edited {
    #[must_use]
    pub fn read(command: &str) -> Vec<Edited> {
        Reading::occurrences(command).into_iter().filter_map(Occurrence::edited).collect()
    }

    #[must_use]
    pub fn edit(&self) -> &Edit {
        &self.edit
    }

    #[must_use]
    pub fn construct(&self) -> Option<Construct> {
        self.construct
    }
}

impl Occurrence {
    fn nested(&self) -> Option<Nested> {
        match self.spotted {
            Spotted::Flagged(flagged) => self.within.first().map(|construct| Nested { flagged, construct: *construct }),
            Spotted::Edit(_) => None,
        }
    }

    fn edited(self) -> Option<Edited> {
        match self.spotted {
            Spotted::Edit(edit) => Some(Edited { edit, construct: self.within.first().copied() }),
            Spotted::Flagged(_) => None,
        }
    }

    fn inside(self, outer: &[Construct]) -> Occurrence {
        Occurrence { within: outer.iter().copied().chain(self.within).collect(), spotted: self.spotted }
    }
}
