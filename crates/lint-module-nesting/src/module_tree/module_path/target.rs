use std::fmt;

use lint_layout::{Name, UsePath, Whole, Word, words};
use lint_placement::CrateRoot;

use super::{ModuleIndex, ModulePath};

#[derive(Debug)]
pub struct Target<'path> {
    crate_root: &'path CrateRoot,
    segments: Vec<&'path Name>,
}

impl<'path> Target<'path> {
    const CRATE: &'static str = "crate";
    const SUPER: &'static str = "super";
    const ANCHORS: [Word<'static>; 3] = words!["crate", "super", "self"];

    pub fn from_use(
        file: &ModulePath<'path>,
        path: &'path UsePath,
        modules: &ModuleIndex<'_>,
    ) -> Option<Target<'path>> {
        let first = path.segments().first()?;
        let start = if *first == Self::CRATE {
            &[]
        } else if *first == Self::SUPER {
            file.parent()?.segments
        } else {
            file.segments
        };
        let anchor = first.matches::<Whole>(&Self::ANCHORS);
        let child: Vec<&Name> = file.segments.iter().chain([first]).collect();
        anchor.map(|_| ()).or_else(|| modules.same_as(file.crate_root, &child).map(|_| ()))?;
        let skip = anchor.into_iter().count();
        let segments = path.segments().iter().skip(skip).try_fold(
            start.iter().collect(),
            |mut segments: Vec<&Name>, segment| {
                if *segment == Self::SUPER {
                    segments.pop().and(Some(segments))
                } else {
                    segments.push(segment);
                    Some(segments)
                }
            },
        )?;
        Some(Target { crate_root: file.crate_root, segments })
    }

    pub fn deepest_in<'file>(&self, modules: &ModuleIndex<'file>) -> Option<ModulePath<'file>> {
        self.prefixes().rev().find_map(|prefix| modules.same_as(self.crate_root, prefix))
    }

    pub fn crossed_directory(&self, user: &ModulePath<'_>, modules: &ModuleIndex<'_>) -> Option<&Target<'path>> {
        self.prefixes()
            .take(self.segments.len())
            .skip(1)
            .find(|boundary| {
                modules.directory_same_as(self.crate_root, boundary).is_some()
                    && self.enclosing(boundary, user).is_none()
            })
            .map(|_| self)
    }

    fn prefixes(&self) -> impl DoubleEndedIterator<Item = &[&'path Name]> {
        (0..=self.segments.len()).filter_map(|length| self.segments.get(..length))
    }

    fn enclosing<'user>(&self, boundary: &[&Name], user: &'user ModulePath<'_>) -> Option<&'user ModulePath<'user>> {
        (self.crate_root == user.crate_root
            && user.segments.get(..boundary.len()).is_some_and(|head| head.iter().eq(boundary.iter().copied())))
        .then_some(user)
    }
}

impl fmt::Display for Target<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}
