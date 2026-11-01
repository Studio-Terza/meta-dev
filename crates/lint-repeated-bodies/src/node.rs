use crate::occurrence::Occurrence;
use body_shape::Shape;

#[derive(Clone, Debug)]
pub(crate) struct Node {
    shape: Shape,
    occurrence: Occurrence,
}

impl Node {
    pub(crate) fn new(shape: Shape, occurrence: Occurrence) -> Node {
        Node { shape, occurrence }
    }

    pub(crate) fn shape(&self) -> Shape {
        self.shape
    }

    pub(crate) fn into_occurrence(self) -> Occurrence {
        self.occurrence
    }
}
