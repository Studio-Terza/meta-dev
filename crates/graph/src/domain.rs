use strum::VariantArray;

#[derive(VariantArray, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Types,
    Macros,
    Values,
}

impl Domain {
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
