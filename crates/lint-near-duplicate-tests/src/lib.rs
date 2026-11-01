mod body_shape;
mod test_bodies;
mod test_body;

use std::collections::BTreeMap;

use body_shape::BodyShape;
use lint_core::{Lint, Report, SourceFile, Violation};
use lint_placement::PackageRoot;
use test_bodies::TestBodies;
use test_body::TestBody;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RepeatKey {
    package: PackageRoot,
    shape: BodyShape,
}

#[derive(Debug)]
pub struct NearDuplicateTests;

impl NearDuplicateTests {
    fn grouped(files: &[SourceFile]) -> BTreeMap<RepeatKey, Vec<TestBody>> {
        files.iter().fold(BTreeMap::new(), |groups, file| {
            TestBodies::found(file).into_iter().fold(groups, |mut groups: BTreeMap<RepeatKey, Vec<TestBody>>, body| {
                let key = RepeatKey { package: file.placement().package().clone(), shape: body.shape().clone() };
                groups.entry(key).or_default().push(body);
                groups
            })
        })
    }

    fn repeats(group: &[TestBody]) -> Vec<Violation> {
        group
            .iter()
            .skip(1)
            .zip(group.first().into_iter().cycle())
            .map(|(repeat, first)| repeat.repeating(first))
            .collect()
    }
}

impl Lint for NearDuplicateTests {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "tests differ by more than their literals",
            Self::grouped(files).values().flat_map(|group| Self::repeats(group)).collect(),
        )
    }
}
