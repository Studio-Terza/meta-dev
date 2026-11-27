use dispatch_ports::{Pause, Wall};
use newtype::{Arbitrary, Deserialize, Newtype, Wrapping};

use crate::Stepped;

#[derive(Newtype, Arbitrary, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(usize);

impl Stepped {
    pub(crate) fn after(now: Wall, pause: Pause) -> Wall {
        Wall::new(now.into_inner().saturating_add(i64::from(pause.into_inner())))
    }
}
