use invariants::Ignoring;

use crate::{Body, Ignored, Line, Unrecognised};

#[derive(Debug)]
pub struct Launch(Body);

impl From<Body> for Launch {
    fn from(tool: Body) -> Launch {
        Launch(tool)
    }
}

impl From<Launch> for Body {
    fn from(launch: Launch) -> Body {
        launch.0
    }
}

#[derive(Debug)]
pub enum Said {
    Text(Body),
    Launched(Launch),
    Silent,
    Unrecognised(Unrecognised),
}

pub trait Hears {
    fn text(self, text: Body) -> Line;

    fn launched(self, launch: Launch) -> Line;

    fn silence(self) -> Line;

    fn unrecognised(self, unrecognised: Unrecognised) -> Line;
}

impl Said {
    pub fn heard<By: Hears>(self, by: By) -> Line {
        match self {
            Said::Text(text) => by.text(text),
            Said::Launched(launch) => by.launched(launch),
            Said::Silent => by.silence(),
            Said::Unrecognised(unrecognised) => by.unrecognised(unrecognised),
        }
    }

    fn joined(self, next: Said) -> Said {
        match self {
            Said::Text(text) => next.after(text),
            Said::Launched(launch) => Said::Launched(launch),
            Said::Silent => next,
            Said::Unrecognised(unrecognised) => Said::Unrecognised(unrecognised),
        }
    }

    fn after(self, before: Body) -> Said {
        match self {
            Said::Text(text) => before.joined(&text).ignoring(Ignored::Oversized).map_or(Said::Silent, Said::Text),
            Said::Launched(launch) => Said::Launched(launch),
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
