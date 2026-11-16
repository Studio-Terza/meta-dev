mod feature_word;
mod finding;
mod flagged;
mod gated_items;
mod offered;
mod test_features;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use feature_word::FeatureWord;
use finding::Finding;
use gated_items::GatedItems;
use graph::Model;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, ModelLint, Parsed, Report};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_placement::PackageRoot;
use lint_sweep::{Gathers, Watchers, Watching};
use offered::{Offer, Used};
use test_features::TestFeatures;

#[derive(Debug)]
pub struct NoTestOnlyApi;

impl NoTestOnlyApi {
    const HEADLINE: Headline = Headline::new("shipped code has no test-only API");

    fn in_manifest<'file>(package: &PackageRoot, file: &'file SourceFile) -> Vec<Violation<'file>> {
        let manifest = file.head().manifest_file();
        package
            .key_lines()
            .features()
            .iter()
            .filter_map(|(name, line)| {
                name.matches::<FeatureWord>(&FeatureWord::TEST_WORDS)
                    .map(|_| Violation::new(Site::new(manifest.as_ref(), *line), Finding::Feature(Cow::Borrowed(name))))
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FileManifest<'head> {
    path: &'head Path,
    package: &'head PackageRoot,
}

#[derive(Debug)]
pub struct TestOnly<'head> {
    manifests: BTreeMap<&'head PackageRoot, &'head Path>,
}

impl<'head> Gathers<FileManifest<'head>> for FileManifest<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Vec::new()
    }

    fn gathered(self) -> FileManifest<'head> {
        self
    }
}

impl AsRef<Path> for FileManifest<'_> {
    fn as_ref(&self) -> &Path {
        self.path
    }
}

impl<'head> FromIterator<FileManifest<'head>> for TestOnly<'head> {
    fn from_iter<Parts: IntoIterator<Item = FileManifest<'head>>>(parts: Parts) -> TestOnly<'head> {
        let manifests =
            parts.into_iter().fold(BTreeMap::new(), |mut last: BTreeMap<&'head PackageRoot, &'head Path>, part| {
                let kept = last.get(part.package).copied().filter(|held| *held > part.path).unwrap_or(part.path);
                last.insert(part.package, kept);
                last
            });
        TestOnly { manifests }
    }
}

impl ContextLint for NoTestOnlyApi {
    const HEADLINE: Headline = NoTestOnlyApi::HEADLINE;
    const READS: Reads = Reads::Every;

    type Part<'head> = FileManifest<'head>;
    type Context<'head> = TestOnly<'head>;

    type Gathering<'head> = FileManifest<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> FileManifest<'head> {
        FileManifest { path: parsed.head().path(), package: parsed.head().placement().package() }
    }
}

impl WatchingLint for NoTestOnlyApi {
    type Found<'file> = Offered<'file>;
    type Watched<'file> = Offered<'file>;

    fn watched(file: &SourceFile) -> Offered<'_> {
        let path = file.head().path();
        let shipped = file.shipped().map(|_| ());
        let gated =
            shipped.into_iter().fold(Watchers::none(), |watchers, ()| watchers.and(Box::new(GatedItems::new(path))));
        let watchers = gated.and(Box::new(TestFeatures::new(path)));
        Offered { watchers, found: Vec::new() }
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Offered<'file>,
        test_only: &TestOnly<'head>,
    ) -> Vec<Violation<'file>> {
        let path = file.head().path();
        let package = file.head().placement().package();
        let manifest =
            test_only.manifests.get(package).filter(|last| **last == path).map(|_| Self::in_manifest(package, file));
        found.found.into_iter().chain(manifest.into_iter().flatten()).collect()
    }
}

#[derive(Debug)]
pub struct Offered<'file> {
    watchers: Watchers<'file>,
    found: Vec<Violation<'file>>,
}

impl<'file> Gathers<Offered<'file>> for Offered<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.watchers.watching()
    }

    fn gathered(self) -> Offered<'file> {
        let found = self.watchers.gathered();
        Offered { watchers: Watchers::none(), found }
    }
}

impl ModelLint for NoTestOnlyApi {
    fn report(model: &Model) -> Report<'_> {
        let used = Used::from(model);
        let offered = Offer::every(model).into_iter().filter_map(|offer| offer.tested_only(&used)).collect();
        Report::new(Self::HEADLINE, offered)
    }
}
