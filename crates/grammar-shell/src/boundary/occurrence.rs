use std::path::Path;

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
pub(super) enum Spotted {
    Flagged(Flagged),
    Edit(Edit),
    Changed(Changed),
    Unreadable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unreadable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Changed {
    To(String),
    Unknown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Directory {
    #[default]
    Started,
    Changed(Vec<String>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Occurrence {
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
    directory: Directory,
}

#[derive(Debug, Default)]
pub(super) struct Walk {
    directory: Directory,
    edited: Vec<Edited>,
}

impl Nested {
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
    pub fn directory(&self) -> &Directory {
        &self.directory
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

impl Walk {
    pub(super) fn stepped(self, occurrence: Occurrence) -> Walk {
        let construct = occurrence.within.first().copied();
        match occurrence.spotted {
            Spotted::Edit(edit) => {
                let edited = Edited { edit, construct, directory: self.directory.clone() };
                Walk { edited: self.edited.into_iter().chain([edited]).collect(), ..self }
            }
            Spotted::Changed(changed) => Walk { directory: self.directory.changed(changed), ..self },
            Spotted::Flagged(_) | Spotted::Unreadable => self,
        }
    }

    pub(super) fn edited(self) -> Vec<Edited> {
        self.edited
    }
}

impl Directory {
    fn changed(self, changed: Changed) -> Directory {
        match (self, changed) {
            (_, Changed::To(step)) if Path::new(&step).is_absolute() => Directory::Changed(vec![step]),
            (Directory::Started, Changed::To(step)) => Directory::Changed(vec![step]),
            (Directory::Changed(steps), Changed::To(step)) => {
                Directory::Changed(steps.into_iter().chain([step]).collect())
            }
            (Directory::Unknown, Changed::To(_)) | (_, Changed::Unknown) => Directory::Unknown,
        }
    }
}

impl Occurrence {
    pub(super) fn new(spotted: Spotted, within: Vec<Construct>) -> Occurrence {
        Occurrence { spotted, within }
    }

    pub(super) fn nested(&self) -> Option<Nested> {
        match self.spotted {
            Spotted::Flagged(flagged) => self.within.first().map(|construct| Nested { flagged, construct: *construct }),
            Spotted::Edit(_) | Spotted::Changed(_) | Spotted::Unreadable => None,
        }
    }

    pub(super) fn unreadable(&self) -> Option<Unreadable> {
        (self.spotted == Spotted::Unreadable).then_some(Unreadable)
    }

    pub(super) fn inside(self, outer: &[Construct]) -> Occurrence {
        Occurrence { within: outer.iter().copied().chain(self.within).collect(), spotted: self.spotted }
    }
}
