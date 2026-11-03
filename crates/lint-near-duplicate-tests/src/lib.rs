mod body_shape;
mod test_bodies;
mod test_body;

use std::collections::BTreeMap;

use body_shape::BodyShape;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Lint};
use lint_placement::PackageRoot;
use test_bodies::TestBodies;
use test_body::Repeat;

#[derive(Debug)]
pub struct NearDuplicateTests;

impl NearDuplicateTests {
    fn grouped(files: &[SourceFile]) -> BTreeMap<&PackageRoot, BTreeMap<BodyShape, Vec<Repeat<'_>>>> {
        files.iter().fold(BTreeMap::new(), |groups, file| {
            TestBodies::found(file).into_iter().fold(groups, |mut groups, body| {
                body.filed(file.placement().package(), &mut groups);
                groups
            })
        })
    }

    fn repeats<'file>(group: &[Repeat<'file>]) -> Vec<Violation<'file>> {
        group
            .iter()
            .skip(1)
            .zip(group.first().into_iter().cycle())
            .map(|(repeat, first)| repeat.repeating(first))
            .collect()
    }
}

impl Lint for NearDuplicateTests {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("tests differ by more than their literals"),
            Self::grouped(files).values().flat_map(BTreeMap::values).flat_map(|group| Self::repeats(group)).collect(),
        )
    }
}
