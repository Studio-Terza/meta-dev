use std::collections::BTreeMap;
use std::fmt;

use code_model::Ground;
use lint_core::Fix;
use lint_placement::CrateName;
use newtype::{Arbitrary, Counter, Display, Newtype};

use super::effect::Effect;
use super::effect_site::EffectSite;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Count(usize);

#[derive(Debug)]
pub(super) struct Summary<'name> {
    name: &'name CrateName,
    total: Count,
    in_ports: Count,
    outside: BTreeMap<Effect, Count>,
}

impl<'name> Summary<'name> {
    pub(super) fn counted(name: &'name CrateName, sites: &[EffectSite<'_>]) -> Summary<'name> {
        let outside = sites.iter().filter(|site| site.ground() == Ground::Open).fold(
            BTreeMap::new(),
            |mut counts: BTreeMap<Effect, Count>, site| {
                let counted = counts.entry(site.effect()).or_default();
                *counted = counted.incremented();
                counts
            },
        );
        Summary {
            name,
            total: Count::new(sites.len()),
            in_ports: Count::new(sites.iter().filter(|site| site.ground() == Ground::Ports).count()),
            outside,
        }
    }
}

impl fmt::Display for Summary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kinds = self
            .outside
            .iter()
            .map(|(effect, count)| format!("{effect} {count}"))
            .reduce(|kinds, kind| format!("{kinds}, {kind}"));
        write!(
            formatter,
            "crate {}: {} effect sites, {} in ports, {} outside ports",
            self.name,
            self.total,
            self.in_ports,
            Count::total(self.outside.values().copied())
        )?;
        kinds.map_or(Ok(()), |listed| write!(formatter, " ({listed})"))
    }
}

impl lint_core::Finding for Summary<'_> {
    fn fix(&self) -> Fix {
        Fix::new("move each effect site outside ports behind a port, until none remain outside")
    }
}
