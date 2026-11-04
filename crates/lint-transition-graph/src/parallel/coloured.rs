use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use equivalence::{Colour, Refinement};

use crate::count::Count;
use crate::end::Node;

#[derive(Debug)]
pub(super) struct Coloured<'found> {
    colours: BTreeMap<Node<'found>, Colour>,
}

impl<'found> Coloured<'found> {
    pub(super) fn refined<Label: Ord, Law: Ord>(refinement: &Refinement<Node<'found>, Label, Law>) -> Coloured<'found> {
        Coloured { colours: refinement.coloured() }
    }

    pub(super) fn colours(&self) -> &BTreeMap<Node<'found>, Colour> {
        &self.colours
    }

    pub(super) fn spread(&self) -> Spread {
        Spread {
            types: Count::new(self.colours.len()),
            colours: Count::new(self.colours.values().collect::<BTreeSet<&Colour>>().len()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Spread {
    types: Count,
    colours: Count,
}

impl fmt::Display for Spread {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} domain types with moves refine into {} colours", self.types, self.colours)
    }
}
