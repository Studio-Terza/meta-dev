use crate::source::Source;

#[derive(Debug)]
pub struct Case {
    name: &'static str,
    sources: &'static [Source],
    flagged: &'static [&'static str],
}

impl Case {
    #[must_use]
    pub const fn new(name: &'static str, sources: &'static [Source], flagged: &'static [&'static str]) -> Case {
        Case { name, sources, flagged }
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn sources(&self) -> &'static [Source] {
        self.sources
    }

    #[must_use]
    pub const fn flagged(&self) -> &'static [&'static str] {
        self.flagged
    }
}
