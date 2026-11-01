use crate::flagged::Flagged;
use crate::source::Source;
use crate::title::Title;

#[derive(Debug)]
pub struct Case {
    name: Title,
    sources: &'static [Source],
    flagged: Flagged,
}

impl Case {
    #[must_use]
    pub const fn new(name: Title, sources: &'static [Source], flagged: Flagged) -> Case {
        Case { name, sources, flagged }
    }

    #[must_use]
    pub const fn name(&self) -> Title {
        self.name
    }

    #[must_use]
    pub const fn sources(&self) -> &'static [Source] {
        self.sources
    }

    #[must_use]
    pub const fn flagged(&self) -> Flagged {
        self.flagged
    }
}
