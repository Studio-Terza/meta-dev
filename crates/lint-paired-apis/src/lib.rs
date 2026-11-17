mod declared;
mod owner;
mod paired_methods;
mod pairs;

use std::collections::BTreeMap;

use algebra::{Monoid, Semigroup};
use declared::Declared;
use lint_core::{Headline, Lint, Parsed, Report, SourceFile};
use lint_file::Gleaning;
use lint_sweep::{Contributing, Gathers, Sweep, Watching};
use owner::Owner;
use paired_methods::PairedMethods;
use pairs::Pairs;

#[derive(Debug)]
pub struct PairedApis;

impl PairedApis {
    const HEADLINE: Headline = Headline::new(
        "report only, a paired API leaves the ending to every caller; write a guard whose Drop does the ending",
    );
}

impl Lint for PairedApis {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::from(&PairedApis::swept(files))
    }
}

#[derive(Debug)]
pub struct PairedOwners<'head>(BTreeMap<Owner<'head>, Vec<Declared<'head>>>);

impl<'head> FromIterator<PairedOwners<'head>> for PairedOwners<'head> {
    fn from_iter<Parts: IntoIterator<Item = PairedOwners<'head>>>(parts: Parts) -> PairedOwners<'head> {
        Monoid::concat(parts)
    }
}

impl Semigroup for PairedOwners<'_> {
    fn combine(mut self, other: Self) -> Self {
        other.0.into_iter().for_each(|(owner, declared)| self.0.entry(owner).or_default().extend(declared));
        self
    }
}

impl Monoid for PairedOwners<'_> {
    fn identity() -> Self {
        PairedOwners(BTreeMap::new())
    }
}

impl<'head> PairedOwners<'head> {
    pub(crate) fn new(owners: BTreeMap<Owner<'head>, Vec<Declared<'head>>>) -> PairedOwners<'head> {
        PairedOwners(owners)
    }
}

impl<'head> From<&PairedOwners<'head>> for Report<'head> {
    fn from(owners: &PairedOwners<'head>) -> Report<'head> {
        Report::new(PairedApis::HEADLINE, Pairs::listed(&owners.0))
    }
}

#[derive(Debug)]
pub struct PairedGleaner<'head>(Contributing<PairedMethods<'head>>);

impl<'head> Gathers<PairedOwners<'head>> for PairedGleaner<'head> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.0)
    }

    fn gathered(self) -> PairedOwners<'head> {
        self.0.concluded(|_, (), owners| owners)
    }
}

impl Gleaning for PairedApis {
    type Part<'head> = PairedOwners<'head>;
    type Gleaned<'head> = PairedOwners<'head>;

    type Gleaner<'run, 'head: 'run> = PairedGleaner<'head>;

    fn gleaner<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> PairedGleaner<'head> {
        PairedGleaner(Contributing::new(PairedMethods::of_file(parsed)))
    }
}
