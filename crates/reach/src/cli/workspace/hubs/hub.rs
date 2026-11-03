use std::cmp::Reverse;

use lint_complexity::{Held, Logic};
use lint_core::Violation;
use newtype::Newtype;

use super::among::Among;
use crate::cli::workspace::crate_name::CrateName;
use crate::cli::workspace::share::Share;

#[derive(Debug)]
pub struct Hub {
    crate_name: CrateName,
    held: Held,
    share: Share,
}

impl Hub {
    pub fn new(crate_name: CrateName, held: Held, share: Share) -> Hub {
        Hub { crate_name, held, share }
    }

    pub fn ranked(mut hubs: Vec<Hub>) -> Vec<Hub> {
        hubs.sort_by(|one, other| {
            (Reverse(one.weight()), &one.crate_name).cmp(&(Reverse(other.weight()), &other.crate_name))
        });
        hubs
    }

    pub fn counted(&self, among: Among) -> Option<Logic> {
        match among {
            Among::All => Some(self.held.logic()),
            Among::Majority => self.share.majority().map(|_| self.held.logic()),
        }
    }

    fn weight(&self) -> Logic {
        Logic::new(self.held.logic().into_inner() * self.share.reached().into_inner())
    }
}

impl From<&Hub> for Violation {
    fn from(hub: &Hub) -> Violation {
        Violation::new(
            hub.held.heaviest().clone(),
            format_args!(
                "crate {}: logic {}, depended on by {}, {}%; weighted {}",
                hub.crate_name,
                hub.held.logic(),
                hub.share,
                hub.share.percent(),
                hub.weight()
            ),
        )
    }
}
