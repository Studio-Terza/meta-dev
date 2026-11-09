mod target;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use serde::Deserialize;

use target::Target;

#[derive(Debug, Deserialize)]
pub(super) struct Package {
    id: String,
    name: String,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
    targets: Vec<Target>,
}

impl Package {
    const DEFAULT: &'static str = "default";

    pub(super) fn among(&self, members: &[String]) -> bool {
        members.contains(&self.id)
    }

    pub(super) fn named(&self, names: &[String]) -> bool {
        names.contains(&self.name)
    }

    pub(super) fn tests(&self) -> impl Iterator<Item = &str> {
        let enabled = self.enabled();
        self.targets.iter().filter(move |target| target.buildable_with(&enabled)).map(Target::name)
    }

    fn enabled(&self) -> BTreeSet<&str> {
        let start: BTreeSet<&str> =
            self.features.keys().map(String::as_str).filter(|name| *name == Self::DEFAULT).collect();
        iter::successors(Some(start), |known| Some(self.implied(known)).filter(|next| next != known))
            .last()
            .unwrap_or_default()
    }

    fn implied<'package>(&'package self, known: &BTreeSet<&'package str>) -> BTreeSet<&'package str> {
        known
            .iter()
            .filter_map(|name| self.features.get(*name))
            .flatten()
            .filter_map(|implied| self.features.get_key_value(implied).map(|(name, _)| name.as_str()))
            .chain(known.iter().copied())
            .collect()
    }
}
