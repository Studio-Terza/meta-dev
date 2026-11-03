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
            Hidden::Nothing => &[],
            Hidden::Singles => &[Quote::Single],
            Hidden::Quoted => &[Quote::Single, Quote::Double],
        }
    }
}

impl From<Key> for Hidden {
    fn from(setting: Key) -> Hidden {
        match setting {
            Key::Shell => Hidden::Nothing,
            Key::Run | Key::Uses | Key::With | Key::Script | Key::Env => Hidden::Quoted,
        }
    }
}
