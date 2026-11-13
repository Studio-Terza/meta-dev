#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Types,
    Macros,
    Values,
}

impl Domain {
    pub const ALL: [Domain; 3] = [Domain::Types, Domain::Macros, Domain::Values];

    pub(crate) fn chosen<Bound>(self, macros: Vec<Bound>, others: Vec<Bound>) -> Vec<Bound> {
        match self {
            Domain::Macros => Domain::preferred(macros, others),
            Domain::Types | Domain::Values => others,
        }
    }

    fn preferred<Bound>(macros: Vec<Bound>, others: Vec<Bound>) -> Vec<Bound> {
        match macros.first().map(|_| ()) {
            None => others,
            Some(()) => macros,
        }
    }
}
