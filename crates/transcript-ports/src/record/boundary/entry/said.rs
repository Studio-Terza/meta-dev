use crate::{Body, Line, Unrecognised};

#[derive(Debug)]
pub enum Said {
    Text(Body),
    Silent,
    Unrecognised(Unrecognised),
}

pub trait Hears {
    fn text(self, text: Body) -> Line;

    fn silence(self) -> Line;

    fn unrecognised(self, unrecognised: Unrecognised) -> Line;
}

impl Said {
    pub fn heard<By: Hears>(self, by: By) -> Line {
        match self {
            Said::Text(text) => by.text(text),
            Said::Silent => by.silence(),
            Said::Unrecognised(unrecognised) => by.unrecognised(unrecognised),
        }
    }

    fn joined(self, next: Said) -> Said {
        match self {
            Said::Text(text) => next.after(text),
            Said::Silent => next,
            Said::Unrecognised(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }

    fn after(self, before: Body) -> Said {
        match self {
            Said::Text(text) => Said::Text(text.after(&before)),
            Said::Silent => Said::Text(before),
            Said::Unrecognised(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }
}

impl FromIterator<Said> for Said {
    fn from_iter<Pieces: IntoIterator<Item = Said>>(pieces: Pieces) -> Said {
        pieces.into_iter().fold(Said::Silent, Said::joined)
    }
}
