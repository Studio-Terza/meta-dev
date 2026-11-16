use std::collections::BTreeSet;
use std::fmt;

use graph::{ItemId, Model, Site};
use lint_core::{Bound, Fix, Violation};
use lint_layout::{Length, Name};

use crate::shaped::Shaped;

#[derive(Debug)]
pub struct Repeated<'model> {
    first: Shaped<'model>,
    owners: Vec<&'model Name>,
}

impl<'model> Repeated<'model> {
    pub fn flagged(model: &'model Model, members: Vec<Shaped<'model>>, most: Bound) -> Option<Violation<'model>> {
        let owners: BTreeSet<ItemId> = members.iter().map(Shaped::owner).collect();
        most.allows(Length::new(owners.len())).map_or(Some(()), |_| None)?;
        let first = members.into_iter().min_by_key(|member| *AsRef::<Site<'model>>::as_ref(member))?;
        let names = owners.into_iter().filter_map(|owner| model.declared(owner)).map(AsRef::<Name>::as_ref).collect();
        Some(Violation::new(*AsRef::<Site<'model>>::as_ref(&first), Repeated { first, owners: names }))
    }
}

impl fmt::Display for Repeated<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owners: Vec<String> = self.owners.iter().map(ToString::to_string).collect();
        write!(formatter, "{} on {}", self.first, owners.join(", "))
    }
}

impl lint_core::Finding for Repeated<'_> {
    fn fix(&self) -> Fix {
        Fix::new("use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive")
    }
}
