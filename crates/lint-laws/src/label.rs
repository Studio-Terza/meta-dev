use std::fmt::{self, Write};
use std::path::Path;

use graph::ItemPath;
use lint_layout::{Name, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Label {
    Generated,
    Monoid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Law<'model> {
    root: &'model Path,
    path: &'model ItemPath,
    label: Label,
}

struct Joined<'parts, Part>(&'parts [Part]);

struct TestName<'model>(&'model ItemPath);

impl Label {
    pub const GENERATES: [Word<'static>; 2] = words!["laws", "generates"];
    pub const MONOID: [Word<'static>; 2] = words!["laws", "monoid"];

    fn monoid(self, path: &ItemPath, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Generated => Ok(()),
            Label::Monoid => writeln!(
                formatter,
                "{}!({}_forms_a_monoid, {}, any::<{}>());",
                Joined(&Label::MONOID),
                TestName(path),
                Joined(path.as_ref()),
                Joined(path.as_ref())
            ),
        }
    }
}

impl<'model> Law<'model> {
    pub fn new(root: &'model Path, path: &'model ItemPath, label: Label) -> Law<'model> {
        Law { root, path, label }
    }
}

impl AsRef<Path> for Law<'_> {
    fn as_ref(&self) -> &Path {
        self.root
    }
}

impl fmt::Display for Law<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter)?;
        writeln!(
            formatter,
            "{}!({}_generates, any::<{}>());",
            Joined(&Label::GENERATES),
            TestName(self.path),
            Joined(AsRef::<[Name]>::as_ref(self.path))
        )?;
        self.label.monoid(self.path, formatter)
    }
}

impl<Part: fmt::Display> fmt::Display for Joined<'_, Part> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(index, part)| {
            (index > 0).then_some("::").map_or(Ok(()), |gap| formatter.write_str(gap))?;
            write!(formatter, "{part}")
        })
    }
}

impl fmt::Display for TestName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        AsRef::<[Name]>::as_ref(self.0).iter().skip(1).enumerate().try_for_each(|(index, segment)| {
            (index > 0).then_some(Self::GAP).map_or(Ok(()), |gap| formatter.write_char(gap))?;
            TestName::segment(segment, formatter)
        })
    }
}

impl TestName<'_> {
    const GAP: char = '_';

    fn segment(segment: &Name, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        AsRef::<str>::as_ref(segment).chars().enumerate().try_for_each(|(at, letter)| {
            (at > 0)
                .then_some(letter)
                .filter(|found| found.is_uppercase())
                .map_or(Ok(()), |_| formatter.write_char(Self::GAP))?;
            letter.to_lowercase().try_for_each(|lowered| formatter.write_char(lowered))
        })
    }
}
