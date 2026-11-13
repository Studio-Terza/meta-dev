mod declared;
mod owner;
mod paired_methods;
mod pairs;

use std::collections::BTreeMap;

use declared::Declared;
use lint_core::{Headline, Lint, Parsed, Report, SourceFile};
use lint_file::Gleaning;
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
        let declared: PairedOwners<'_> = files.iter().map(Parsed::from).map(PairedApis::gleaned).collect();
        Report::from(&declared)
    }
}

#[derive(Debug, Default)]
pub struct PairedOwners<'head>(BTreeMap<Owner<'head>, Vec<Declared<'head>>>);

impl<'head> FromIterator<PairedOwners<'head>> for PairedOwners<'head> {
    fn from_iter<Parts: IntoIterator<Item = PairedOwners<'head>>>(parts: Parts) -> PairedOwners<'head> {
        PairedOwners(parts.into_iter().flat_map(|part| part.0).fold(
            BTreeMap::new(),
            |mut owners, (owner, declared)| {
                owners.entry(owner).or_insert_with(Vec::new).extend(declared);
                owners
            },
        ))
    }
}

impl<'head> From<&PairedOwners<'head>> for Report<'head> {
    fn from(owners: &PairedOwners<'head>) -> Report<'head> {
        Report::new(PairedApis::HEADLINE, Pairs::listed(&owners.0))
    }
}

impl Gleaning for PairedApis {
    type Part<'head> = PairedOwners<'head>;
    type Gleaned<'head> = PairedOwners<'head>;

    fn gleaned<'head>(parsed: Parsed<'_, 'head>) -> PairedOwners<'head> {
        PairedOwners(parsed.file().shipped().map_or_default(|_| PairedMethods::of_file(parsed)))
    }
}
