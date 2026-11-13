mod declarations;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Parsed, SourceFile, Violation};
use lint_effect_sites::CrateEffects;
use lint_layout::Name;
use lint_paths::Imports;
use lint_placement::CrateName;

use declarations::Declarations;

#[derive(Debug)]
pub struct PortedCrates<'head> {
    imports: Imports<'head>,
    ports: BTreeMap<CrateName, BTreeSet<Name>>,
}

impl<'head> From<Parsed<'_, 'head>> for PortedCrates<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> PortedCrates<'head> {
        let shipped = parsed.file().shipped().map(|_| parsed);
        let ports = shipped.map(|found| {
            (CrateName::from(found.head().placement().package().root()), Declarations::ports(found.file()))
        });
        PortedCrates { imports: shipped.into_iter().map(Imports::from).collect(), ports: ports.into_iter().collect() }
    }
}

#[derive(Default)]
struct Gathered<'head> {
    imports: Vec<Imports<'head>>,
    ports: BTreeMap<CrateName, BTreeSet<Name>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: PortedCrates<'head>) -> Gathered<'head> {
        self.imports.push(part.imports);
        part.ports.into_iter().for_each(|(crate_name, ports)| self.ports.entry(crate_name).or_default().extend(ports));
        self
    }
}

impl<'head> FromIterator<PortedCrates<'head>> for PortedCrates<'head> {
    fn from_iter<Parts: IntoIterator<Item = PortedCrates<'head>>>(parts: Parts) -> PortedCrates<'head> {
        let gathered = parts.into_iter().fold(Gathered::default(), Gathered::including);
        PortedCrates { imports: gathered.imports.into_iter().collect(), ports: gathered.ports }
    }
}

impl PortedCrates<'_> {
    pub(crate) fn stray_effects<'file>(&self, file: &'file SourceFile) -> Vec<Violation<'file>> {
        let crate_name = CrateName::from(file.head().placement().package().root());
        let ports = self.ports.get(&crate_name).filter(|declared| declared.first().is_some());
        ports.map_or_default(|declared| {
            CrateEffects::found_in(file, &self.imports).outside_fillers(&Declarations::fillers(file, declared))
        })
    }
}
