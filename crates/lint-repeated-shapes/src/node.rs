use crate::occurrence::Occurrence;
use body_shape::Shape;

#[derive(Clone, Debug)]
pub(crate) struct Node<'file> {
    shape: Shape,
    occurrence: Occurrence<'file>,
}

impl<'file> Node<'file> {
    pub(crate) fn new(shape: Shape, occurrence: Occurrence<'file>) -> Node<'file> {
        Node { shape, occurrence }
    }

    pub(crate) fn shape(&self) -> Shape {
        self.shape
    }

    pub(crate) fn into_occurrence(self) -> Occurrence<'file> {
        self.occurrence
    }
}
