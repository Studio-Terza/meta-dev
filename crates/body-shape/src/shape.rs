mod labels;
mod literals;
mod shape_key;
mod spelling;

use crate::token_count::TokenCount;
pub(crate) use shape_key::ShapeKey;

pub(crate) use labels::Code;
pub use labels::Labels;
pub use literals::Literals;
pub(crate) use spelling::{Sign, Spelled};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shape {
    size: TokenCount,
    key: ShapeKey,
}

impl Shape {
    #[must_use]
    pub fn size(self) -> TokenCount {
        self.size
    }
}

impl From<&Labels> for Shape {
    fn from(labels: &Labels) -> Shape {
        Shape { size: TokenCount::new(labels.count()), key: ShapeKey::from(labels) }
    }
}

#[derive(Clone, Debug)]
pub struct Reading {
    shape: Shape,
    literals: Literals,
    labels: Labels,
}

impl Reading {
    pub(crate) fn new(literals: Literals, labels: Labels) -> Reading {
        Reading { shape: Shape::from(&labels), literals, labels }
    }

    #[must_use]
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    #[must_use]
    pub fn shape(&self) -> Shape {
        self.shape
    }

    #[must_use]
    pub fn into_literals(self) -> Literals {
        self.literals
    }
}
