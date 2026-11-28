mod lexeme;
mod reading;
mod walk;

use std::fmt;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plain {
    unruled: Vec<Deletion>,
    edits: Vec<Edited>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    span: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deletion {
    Rm,
    Rmdir,
    FindDelete,
    CargoClean,
    GitClean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    Sed,
    Perl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    InPlace(Editor),
    Wrote(Target),
    Moved { sources: Vec<Target>, destination: Target },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    text: String,
    parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    edit: Edit,
    directory: Directory,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Directory {
    #[default]
    Started,
    Changed(Vec<Target>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Part {
    Text(String),
    Variable(String),
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Changed {
    To(Target),
    Unknown,
}

#[derive(Clone, Debug, Default)]
struct Simple {
    words: Vec<Word>,
    written: Vec<Word>,
    scripts: Vec<Vec<Simple>>,
}

#[derive(Clone, Debug, Default)]
struct Word {
    text: String,
    quoting: Quoting,
    expanding: Expanding,
    parts: Vec<Part>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Quoting {
    #[default]
    Plain,
    Quoted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Expanding {
    #[default]
    Literal,
    Expanded,
}

#[derive(Debug)]
struct Prefixed<'text, Token> {
    token: Token,
    rest: &'text str,
}

impl Plain {
    #[must_use]
    pub fn unruled(&self) -> &[Deletion] {
        &self.unruled
    }

    #[must_use]
    pub fn edits(&self) -> &[Edited] {
        &self.edits
    }
}

impl Unreadable {
    #[must_use]
    pub fn span(&self) -> &str {
        &self.span
    }
}

impl Edited {
    #[must_use]
    pub fn directory(&self) -> &Directory {
        &self.directory
    }

    #[must_use]
    pub fn edit(&self) -> &Edit {
        &self.edit
    }

    fn beneath(self, outer: &Directory) -> Edited {
        let directory = match self.directory {
            Directory::Started => outer.clone(),
            Directory::Changed(steps) => {
                steps.into_iter().fold(outer.clone(), |directory, step| directory.changed(Changed::To(step)))
            }
            Directory::Unknown => Directory::Unknown,
        };
        Edited { edit: self.edit, directory }
    }
}

impl Target {
    #[must_use]
    pub fn resolved(&self, variable: impl Fn(&str) -> Option<String>) -> Option<String> {
        self.parts
            .iter()
            .map(|part| match part {
                Part::Text(text) => Some(text.clone()),
                Part::Variable(name) => variable(name),
                Part::Opaque => None,
            })
            .collect()
    }

    fn opaque(&self) -> Option<()> {
        self.parts.contains(&Part::Opaque).then_some(())
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl Directory {
    fn changed(self, changed: Changed) -> Directory {
        match (self, changed) {
            (_, Changed::To(step)) if Path::new(&step.text).is_absolute() => Directory::Changed(vec![step]),
            (Directory::Started, Changed::To(step)) => Directory::Changed(vec![step]),
            (Directory::Changed(steps), Changed::To(step)) => {
                Directory::Changed(steps.into_iter().chain([step]).collect())
            }
            (Directory::Unknown, Changed::To(_)) | (_, Changed::Unknown) => Directory::Unknown,
        }
    }
}
