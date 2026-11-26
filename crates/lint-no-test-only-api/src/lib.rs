mod feature_word;
mod finding;
mod flagged;
mod gated_items;
mod test_features;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use feature_word::FeatureWord;
use finding::Finding;
use gated_items::GatedItems;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_placement::PackageRoot;
use lint_sweep::{Contributing, Finds, Gathers};
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

#[derive(Debug)]
pub struct ManifestGathering<'head>(FileManifest<'head>);

impl<'head> Gathers<FileManifest<'head>> for ManifestGathering<'head> {
    fn gathered(self, _: &SourceFile) -> FileManifest<'head> {
        self.0
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
                last.entry(part.package).and_modify(|held| *held = (*held).max(part.path)).or_insert(part.path);
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

    type Gathering<'head> = ManifestGathering<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> ManifestGathering<'head> {
        ManifestGathering(FileManifest {
            path: AsRef::<Path>::as_ref(parsed.head()),
            package: parsed.head().placement().package(),
        })
    }
}

impl WatchingLint for NoTestOnlyApi {
    type Found<'file> = Offered<'file>;
    type Watched<'file> = Offering<'file>;

    fn watched(file: &SourceFile) -> Offering<'_> {
        let path = AsRef::<Path>::as_ref(file.head());
        let shipped = file.head().shipped().map(|_| ());
        let gated = shipped.map(|()| Contributing::new(GatedItems::from(path)));
        Offering { gated, features: Contributing::new(TestFeatures::from(path)) }
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Offered<'file>,
        test_only: &TestOnly<'head>,
    ) -> Vec<Violation<'file>> {
        let path = AsRef::<Path>::as_ref(file.head());
        let package = file.head().placement().package();
        let manifest =
            test_only.manifests.get(package).filter(|last| **last == path).map(|_| Self::in_manifest(package, file));
        found.found.into_iter().chain(manifest.into_iter().flatten()).collect()
    }
}

#[derive(Debug)]
pub struct Offering<'file> {
    gated: Option<Contributing<GatedItems<'file>>>,
    features: Contributing<TestFeatures<'file>>,
}

#[derive(Debug)]
pub struct Offered<'file> {
    found: Vec<Violation<'file>>,
}

impl<'file> Gathers<Offered<'file>> for Offering<'file> {
    fn gathered(self, file: &SourceFile) -> Offered<'file> {
        let gated = self.gated.into_iter().flat_map(|watcher| watcher.found(file));
        Offered { found: gated.chain(self.features.found(file)).collect() }
    }
}
