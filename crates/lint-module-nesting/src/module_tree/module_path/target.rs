use std::fmt;

use lint_layout::{Name, UsePath, Whole, Word, words};
use lint_placement::CrateRoot;

use super::{ModuleIndex, ModulePath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Start {
    Crate,
    Super,
    Relative,
}

impl Start {
    const NAMED: [(&'static str, Start); 2] = [("crate", Start::Crate), ("super", Start::Super)];

    fn spelled(name: &Name) -> Start {
        Self::NAMED.iter().find(|(word, _)| *name == *word).map_or(Start::Relative, |(_, start)| *start)
    }

    fn advance<'path>(self, mut segments: Vec<&'path Name>, segment: &'path Name) -> Option<Vec<&'path Name>> {
        match self {
            Start::Super => segments.pop().and(Some(segments)),
            Start::Crate | Start::Relative => Some(segments.into_iter().chain([segment]).collect()),
        }
    }
}

#[derive(Debug)]
pub struct Target<'path> {
    crate_root: &'path CrateRoot,
    segments: Vec<&'path Name>,
}

impl<'path> Target<'path> {
    const ANCHORS: [Word<'static>; 3] = words!["crate", "super", "self"];

    pub fn from_use(
        file: &ModulePath<'path>,
        path: &'path UsePath,
        modules: &ModuleIndex<'_>,
    ) -> Option<Target<'path>> {
        let first = path.segments().first()?;
        let start = Self::start(Start::spelled(first), file)?;
        let anchor = first.matches::<Whole>(&Self::ANCHORS);
        let child: Vec<&Name> = file.segments.iter().chain([first]).collect();
        anchor.map(|_| ()).or_else(|| modules.same_as(file.crate_root, &child).map(|_| ()))?;
        let skip = anchor.into_iter().count();
        let segments =
            path.segments().iter().skip(skip).try_fold(start.iter().collect(), |segments, segment| {
                Start::spelled(segment).advance(segments, segment)
            })?;
        Some(Target { crate_root: file.crate_root, segments })
    }

    pub fn deepest_in<'file>(&self, modules: &ModuleIndex<'file>) -> Option<ModulePath<'file>> {
        self.prefixes().rev().find_map(|prefix| modules.same_as(self.crate_root, prefix))
    }

    pub fn crossed_directory(&self, user: &ModulePath<'_>, modules: &ModuleIndex<'_>) -> Option<&Target<'path>> {
        self.prefixes()
            .take(self.segments.len())
            .skip(1)
            .filter(|boundary| modules.directory_same_as(self.crate_root, boundary).is_some())
            .find(|boundary| self.enclosing(boundary, user).is_none())
            .map(|_| self)
    }

    fn prefixes(&self) -> impl DoubleEndedIterator<Item = &[&'path Name]> {
        (0..=self.segments.len()).filter_map(|length| self.segments.get(..length))
    }

    fn enclosing<'user>(&self, boundary: &[&Name], user: &'user ModulePath<'_>) -> Option<&'user ModulePath<'user>> {
        Some(user).filter(|user| self.crate_root == user.crate_root).filter(|user| {
            user.segments.get(..boundary.len()).is_some_and(|head| head.iter().eq(boundary.iter().copied()))
        })
    }

    fn start<'file>(start: Start, file: &ModulePath<'file>) -> Option<&'file [Name]> {
        match start {
            Start::Crate => Some(&[]),
            Start::Super => file.parent().map(|parent| parent.segments),
            Start::Relative => Some(file.segments),
        }
    }
}

impl fmt::Display for Target<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}
