use std::fmt;

use crate::{Admitted, Complexity, Scale};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Growth {
    class: Complexity,
    factor: Scale,
    admitted: Vec<Admitted>,
}

impl Growth {
    pub(crate) fn new(class: Complexity, factor: Scale, admitted: Vec<Admitted>) -> Growth {
        Growth { class, factor, admitted }
    }
}

impl fmt::Display for Growth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "  {:?} at scale {}:", self.class, self.factor)?;
        self.admitted.iter().try_for_each(|admitted| write!(formatter, " {admitted};"))
    }
}
