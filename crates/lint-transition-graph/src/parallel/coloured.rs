use equivalence::{Colour, Keyed};

use crate::end::Node;

#[derive(Clone, Copy, Debug)]
pub(super) struct Coloured<'found> {
    node: Node<'found>,
    colour: Colour,
}

impl<'found> Coloured<'found> {
    pub(super) fn new(node: Node<'found>, colour: Colour) -> Coloured<'found> {
        Coloured { node, colour }
    }

    pub(super) fn node(self) -> Node<'found> {
        self.node
    }
}

impl Keyed for Coloured<'_> {
    type Key = Colour;

    fn key(&self) -> Colour {
        self.colour
    }
}
