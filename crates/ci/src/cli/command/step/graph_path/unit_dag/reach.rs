use std::collections::BTreeMap;

use super::super::unit_key::UnitKey;
use super::super::weighed::Weighed;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug)]
pub(super) struct Reach {
    seconds: Seconds,
    via: Option<UnitKey>,
}

impl AsRef<Seconds> for Reach {
    fn as_ref(&self) -> &Seconds {
        &self.seconds
    }
}

impl Reach {
    pub(super) fn after(longest: Option<Weighed>, weight: Seconds) -> Reach {
        let before = longest.as_ref().map_or_else(Seconds::default, |unit| *AsRef::<Seconds>::as_ref(unit));
        Reach { seconds: [before, weight].into_iter().sum(), via: longest.map(Weighed::into_key) }
    }

    pub(super) fn via(&self) -> Option<UnitKey> {
        self.via.clone()
    }

    pub(super) fn kept(self, key: &UnitKey, reached: &mut BTreeMap<UnitKey, Reach>) -> Seconds {
        let seconds = self.seconds;
        reached.insert(key.clone(), self);
        seconds
    }
}
