use std::cmp::Reverse;
use std::fmt;

use lint_complexity::{Held, Logic};
use lint_core::{Finding, Fix, Violation};
use newtype::Newtype;

use super::among::Among;
use crate::cli::request::workspace::crate_name::CrateName;
use crate::cli::request::workspace::share::Share;

#[derive(Debug)]
pub struct Hub<'meta, 'file> {
    crate_name: &'meta CrateName,
    held: Held<'file>,
    share: Share,
}

impl<'meta, 'file> Hub<'meta, 'file> {
    pub fn new(crate_name: &'meta CrateName, held: Held<'file>, share: Share) -> Hub<'meta, 'file> {
        Hub { crate_name, held, share }
    }

    pub fn ranked(mut hubs: Vec<Hub<'meta, 'file>>) -> Vec<Hub<'meta, 'file>> {
        hubs.sort_by(|one, other| {
            (Reverse(one.weight()), one.crate_name).cmp(&(Reverse(other.weight()), other.crate_name))
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

impl fmt::Display for Hub<'_, '_> {
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

impl Finding for Hub<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("move the logic into leaf crates and keep types, traits and interfaces here")
    }
}

impl<'file> From<&Hub<'_, 'file>> for Violation<'file> {
    fn from(hub: &Hub<'_, 'file>) -> Violation<'file> {
        Violation::new(hub.held.heaviest(), hub)
    }
}
