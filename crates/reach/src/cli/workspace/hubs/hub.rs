use std::cmp::Reverse;
use std::fmt;

use lint_complexity::{Held, Logic};
use lint_core::{Finding, Fix, Violation};
use newtype::Newtype;

use super::among::Among;
use crate::cli::workspace::crate_name::CrateName;
use crate::cli::workspace::share::Share;

#[derive(Debug)]
pub struct Hub<'file> {
    crate_name: CrateName,
    held: Held<'file>,
    share: Share,
}

impl<'file> Hub<'file> {
    pub fn new(crate_name: CrateName, held: Held<'file>, share: Share) -> Hub<'file> {
        Hub { crate_name, held, share }
    }

    pub fn ranked(mut hubs: Vec<Hub<'file>>) -> Vec<Hub<'file>> {
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

impl fmt::Display for Hub<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "crate {}: logic {}, depended on by {}, {}%; weighted {}",
            self.crate_name,
            self.held.logic(),
            self.share,
            self.share.percent(),
            self.weight()
        )
    }
}

impl Finding for Hub<'_> {
    fn fix(&self) -> Fix {
        Fix::new("move the logic into leaf crates and keep types, traits and interfaces here")
    }
}

impl<'file> From<&Hub<'file>> for Violation<'file> {
    fn from(hub: &Hub<'file>) -> Violation<'file> {
        Violation::new(hub.held.heaviest(), hub)
    }
}
