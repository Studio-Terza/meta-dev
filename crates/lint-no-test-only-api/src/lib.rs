mod candidate;
mod feature_word;
mod finding;
mod flagged;
mod gated_items;
mod mentions;
mod offers;
mod test_features;

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::Path;

use candidate::Candidate;
use feature_word::FeatureWord;
use finding::Finding;
use gated_items::GatedItems;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_layout::Name;
use lint_placement::PackageRoot;
use lint_sweep::{Gathers, Sweep, Watching};
use mentions::Mentions;
use offers::Offers;
use test_features::TestFeatures;

#[derive(Debug)]
pub struct NoTestOnlyApi;

impl NoTestOnlyApi {
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

#[derive(Debug)]
pub struct FileMentions<'head> {
    path: &'head Path,
    package: &'head PackageRoot,
    shipped: BTreeSet<Name>,
    tested: BTreeSet<Name>,
}

#[derive(Debug)]
pub struct TestOnly<'head> {
    names: BTreeSet<Name>,
    manifests: BTreeMap<&'head PackageRoot, &'head Path>,
}

#[derive(Debug)]
pub struct MentionGathering<'head> {
    path: &'head Path,
    package: &'head PackageRoot,
    mentions: Mentions,
}

impl<'head> Gathers<FileMentions<'head>> for MentionGathering<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(&mut self.mentions)
    }

    fn gathered(self) -> FileMentions<'head> {
        self.mentions.gathered().mentioned(self.path, self.package)
    }
}

impl AsRef<Path> for FileMentions<'_> {
    fn as_ref(&self) -> &Path {
        self.path
    }
}

impl<'head> FromIterator<FileMentions<'head>> for TestOnly<'head> {
    fn from_iter<Parts: IntoIterator<Item = FileMentions<'head>>>(parts: Parts) -> TestOnly<'head> {
        let mentions: Vec<FileMentions<'head>> = parts.into_iter().collect();
        let shipped: BTreeSet<&Name> = mentions.iter().flat_map(|part| &part.shipped).collect();
        let names = mentions
            .iter()
            .flat_map(|part| &part.tested)
            .filter(|name| shipped.contains(name).then_some(()).is_none())
            .cloned()
            .collect();
        let manifests =
            mentions.iter().fold(BTreeMap::new(), |mut last: BTreeMap<&'head PackageRoot, &'head Path>, part| {
                let kept = last.get(part.package).copied().filter(|held| *held > part.path).unwrap_or(part.path);
                last.insert(part.package, kept);
                last
            });
        TestOnly { names, manifests }
    }
}

impl ContextLint for NoTestOnlyApi {
    const HEADLINE: Headline = Headline::new("shipped code has no test-only API");
    const READS: Reads = Reads::Every;

    type Part<'head> = FileMentions<'head>;
    type Context<'head> = TestOnly<'head>;

    type Gathering<'head> = MentionGathering<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> MentionGathering<'head> {
        MentionGathering {
            path: parsed.head().path(),
            package: parsed.head().placement().package(),
            mentions: Mentions::of_file(parsed.file()),
        }
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, test_only: &TestOnly<'head>) -> Vec<Violation<'file>> {
        let only_tests = Offers::in_file(file)
            .into_iter()
            .filter(|candidate| test_only.names.contains(candidate.called()))
            .map(Candidate::violation);
        let gated =
            iter::once(file).filter_map(SourceFile::shipped).flat_map(|code| GatedItems::in_file(code.syntax()));
        let flagged = gated.chain(TestFeatures::in_file(file.syntax())).map(|found| found.in_file(file.head().path()));
        let package = file.head().placement().package();
        let manifest = test_only
            .manifests
            .get(package)
            .filter(|last| **last == file.head().path())
            .map(|_| Self::in_manifest(package, file));
        flagged.chain(only_tests).chain(manifest.into_iter().flatten()).collect()
    }
}
