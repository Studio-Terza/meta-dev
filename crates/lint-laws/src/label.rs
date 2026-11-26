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

struct Snaked<'name>(&'name Name);

struct Labelled<'model> {
    label: Label,
    path: &'model ItemPath,
}

impl Label {
    pub const GENERATES: [Word<'static>; 2] = words!["laws", "generates"];
    pub const MONOID: [Word<'static>; 2] = words!["laws", "monoid"];
}

impl fmt::Display for Labelled<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.label {
            Label::Generated => Ok(()),
            Label::Monoid => writeln!(
                formatter,
                "{}!({}_forms_a_monoid, {}, any::<{}>());",
                Joined(&Label::MONOID),
                TestName(self.path),
                Joined(self.path.as_ref()),
                Joined(self.path.as_ref())
            ),
        }
    }
}

impl<'model> Law<'model> {
    pub fn new(root: &'model Path, path: &'model ItemPath, label: Label) -> Law<'model> {
        Law { root, path, label }
    }
}

impl<'model> Law<'model> {
    pub fn home(self) -> &'model Path {
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
        write!(formatter, "{}", Labelled { label: self.label, path: self.path })
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
            write!(formatter, "{}", Snaked(segment))
        })
    }
}

impl TestName<'_> {
    const GAP: char = '_';
}

impl fmt::Display for Snaked<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        AsRef::<str>::as_ref(self.0).chars().enumerate().try_for_each(|(at, letter)| {
            (at > 0)
                .then_some(letter)
                .filter(|found| found.is_uppercase())
                .map_or(Ok(()), |_| formatter.write_char(TestName::GAP))?;
            letter.to_lowercase().try_for_each(|lowered| formatter.write_char(lowered))
        })
    }
}
