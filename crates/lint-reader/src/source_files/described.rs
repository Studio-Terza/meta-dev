use lint_layout::Crates;
use workspace_metadata::Metadata;

#[derive(Debug)]
pub struct Described {
    crates: Crates,
    workspaces: Vec<Metadata>,
}

impl Described {
    pub(super) fn new(crates: Crates, workspaces: Vec<Metadata>) -> Described {
        Described { crates, workspaces }
    }

    #[must_use]
    pub fn crates(&self) -> &Crates {
        &self.crates
    }

    #[must_use]
    pub fn workspaces(&self) -> &[Metadata] {
        &self.workspaces
    }
}
