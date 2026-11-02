mod candidate;
mod feature_word;
mod finding;
mod flagged;
mod gated_items;
mod mentions;
mod offers;
mod test_features;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use candidate::Candidate;
use feature_word::FeatureWord;
use finding::Finding;
use gated_items::GatedItems;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Lint};
use lint_layout::Name;
use lint_placement::PackageRoot;
use mentions::{Ground, Mentions};
use offers::Offers;
use test_features::TestFeatures;

#[derive(Debug)]
pub struct NoTestOnlyApi;

impl NoTestOnlyApi {
    fn in_source(file: &SourceFile) -> Vec<Violation> {
        let shipped =
            iter::once(file).filter(|file| file.is_shipped()).flat_map(|file| GatedItems::in_file(file.syntax()));
        shipped.chain(TestFeatures::in_file(file.syntax())).map(|flagged| flagged.in_file(file.path())).collect()
    }

    fn mentioned(files: &[SourceFile], ground: Ground) -> BTreeSet<Name> {
        files.iter().flat_map(|file| Mentions::in_file(file, ground)).collect()
    }

    fn in_manifest(package: &PackageRoot, file: &SourceFile) -> Vec<Violation> {
        let manifest = file.manifest_file();
        package
            .features()
            .iter()
            .filter(|name| FeatureWord::is_test(name))
            .map(|name| Violation::new(Site::of_file(&manifest), Finding::Feature(name.clone())))
            .collect()
    }
}

impl Lint for NoTestOnlyApi {
    fn report(files: &[SourceFile]) -> Report {
        let packages: BTreeMap<&PackageRoot, &SourceFile> =
            files.iter().rev().map(|file| (file.placement().package(), file)).collect();
        let shipped = Self::mentioned(files, Ground::Shipped);
        let tested = Self::mentioned(files, Ground::Tests);
        let only_tests = files
            .iter()
            .flat_map(Offers::in_file)
            .filter(|candidate| !shipped.contains(candidate.called()) && tested.contains(candidate.called()))
            .map(Candidate::violation);
        Report::new(
            Headline::new("shipped code has no test-only API"),
            files
                .iter()
                .flat_map(Self::in_source)
                .chain(only_tests)
                .chain(packages.into_iter().flat_map(|(package, file)| Self::in_manifest(package, file)))
                .collect(),
        )
    }
}
