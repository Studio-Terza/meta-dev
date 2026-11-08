mod boundary;

use design_model::{NotesLine, Project, Title};
use port_core::Op;

#[derive(Clone, Debug, Op)]
pub struct ImportedNotes(Project);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Noted {
    line: NotesLine,
    title: Title,
}

impl From<Project> for ImportedNotes {
    fn from(project: Project) -> ImportedNotes {
        ImportedNotes(project)
    }
}

impl Noted {
    #[must_use]
    pub fn line(&self) -> NotesLine {
        self.line
    }

    #[must_use]
    pub fn title(&self) -> &Title {
        &self.title
    }
}
