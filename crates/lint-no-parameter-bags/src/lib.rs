mod bag_candidates;
mod name_uses;
mod root_exports;

use bag_candidates::BagCandidates;
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{ContextLint, Reads};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Gathers, Sweep, Watching};
use name_uses::NameUses;
use root_exports::RootExports;

#[derive(Debug)]
pub struct NoParameterBags;

#[derive(Debug)]
pub struct BagUses<'head> {
    uses: Option<NameUses<'head>>,
    exports: RootExports,
}

impl<'head> Gathers<BagUses<'head>> for BagUses<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.uses.as_mut().map_or_default(|uses| Sweep::alone(uses))
    }

    fn gathered(self) -> BagUses<'head> {
        self
    }
}

#[derive(Debug)]
pub struct Bags<'head> {
    uses: Vec<NameUses<'head>>,
    exports: RootExports,
}

impl<'head> FromIterator<BagUses<'head>> for Bags<'head> {
    fn from_iter<Parts: IntoIterator<Item = BagUses<'head>>>(parts: Parts) -> Bags<'head> {
        let (uses, exports): (Vec<Option<NameUses<'head>>>, Vec<RootExports>) =
            parts.into_iter().map(|part| (part.uses, part.exports)).unzip();
        Bags { uses: uses.into_iter().flatten().collect(), exports: exports.into_iter().collect() }
    }
}

impl Bags<'_> {
    fn one_consumer(&self, file: &SourceFile, name: &Name) -> Option<&NameUses<'_>> {
        let crate_name = CrateName::from(file.head().placement().crate_root());
        let consumers: Vec<&NameUses<'_>> =
            self.uses.iter().filter_map(|uses| uses.consumes(file.head().path(), &crate_name, name)).collect();
        Some(consumers).filter(|candidates| candidates.len() == 1).and_then(|sole| sole.into_iter().next())
    }
}

impl ContextLint for NoParameterBags {
    const HEADLINE: Headline =
        Headline::new("a struct with no logic of its own that one other file consumes is a parameter bag");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = BagUses<'head>;
    type Context<'head> = Bags<'head>;

    type Gathering<'head> = BagUses<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> BagUses<'head> {
        let shipped = parsed.file().shipped().map(|_| parsed);
        BagUses { uses: shipped.map(NameUses::from), exports: shipped.map_or_default(RootExports::from) }
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, bags: &Bags<'head>) -> Vec<Violation<'file>> {
        let candidates = BagCandidates::from(file);
        candidates
            .candidates()
            .filter(|candidate| bags.exports.exported(file, candidate.name()).is_none())
            .filter(|candidate| bags.one_consumer(file, candidate.name()).is_some())
            .map(|candidate| candidate.violation(file.head().path()))
            .collect()
    }
}
