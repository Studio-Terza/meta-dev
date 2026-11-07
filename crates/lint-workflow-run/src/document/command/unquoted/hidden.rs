use super::quote::Quote;
use crate::document::key::Key;

#[derive(Clone, Copy, Debug)]
pub(in crate::document::command) enum Hidden {
    Nothing,
    Singles,
    Quoted,
}

impl Hidden {
    pub(super) fn keeps(self, before: Quote, after: Quote) -> Option<()> {
        self.hides()
            .iter()
            .find(|hidden| **hidden == before)
            .map_or(Some(()), |_| None)
            .and_then(|()| (before == after).then_some(()))
    }

    fn hides(self) -> &'static [Quote] {
        match self {
            Self::Nothing => &[],
            Self::Singles => &[Quote::Single],
            Self::Quoted => &[Quote::Single, Quote::Double],
        }
    }
}

impl From<Key> for Hidden {
    fn from(setting: Key) -> Self {
        match setting {
            Key::Shell => Self::Nothing,
            Key::Run | Key::Uses | Key::With | Key::Script | Key::Env => Self::Quoted,
        }
    }
}
