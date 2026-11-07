#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Types,
    Macros,
}

impl Domain {
    pub const ALL: [Domain; 2] = [Domain::Types, Domain::Macros];
}
