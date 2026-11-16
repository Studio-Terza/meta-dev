use super::{Depth, Number};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Break,
    Stop,
    Numbered(Number, Depth),
    Bulleted,
    Quoted,
    Uuid,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    Line,
    Sentence,
}

pub trait Opened {
    const BULLET: Self;

    const QUOTE: Self;

    fn item(number: Number, depth: Depth) -> Self;
}

impl Mark {
    pub fn opened<O: Opened>(self) -> Option<O> {
        match self {
            Mark::Numbered(number, depth) => Some(O::item(number, depth)),
            Mark::Bulleted => Some(O::BULLET),
            Mark::Quoted => Some(O::QUOTE),
            Mark::Break | Mark::Stop | Mark::Uuid | Mark::Plain => None,
        }
    }

    #[must_use]
    pub fn boundary(self) -> Option<Boundary> {
        match self {
            Mark::Break => Some(Boundary::Line),
            Mark::Stop => Some(Boundary::Sentence),
            Mark::Numbered(..) | Mark::Bulleted | Mark::Quoted | Mark::Uuid | Mark::Plain => None,
        }
    }
}
