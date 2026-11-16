use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use named_derive::Named;
use text_lines::Lines;

use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub(in super::super) struct Tested(BTreeSet<PathBuf>);

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum Reaching {
    #[name("changed")]
    Changed,
    #[name("consumed-by")]
    ConsumedBy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum Reach {
    Changed,
    ConsumedBy(PathBuf),
}

impl Reach {
    pub(in super::super) fn taken(reaching: Reaching, arguments: &mut Arguments) -> Result<Reach, Failure<'static>> {
        match reaching {
            Reaching::Changed => Ok(Reach::Changed),
            Reaching::ConsumedBy => Ok(Reach::ConsumedBy(PathBuf::from(arguments.taken()?))),
        }
    }

    fn consumed(&self, ports: &impl Ports, files: &[String]) -> Result<Vec<String>, Failure<'static>> {
        match self {
            Reach::Changed => Ok(Vec::new()),
            Reach::ConsumedBy(lint) => Tested::consumed(ports, lint, files),
        }
    }
}

impl Tested {
    const MARKS: [&'static str; 2] = ["src", "tests"];
    const QUERY: [&'static str; 3] = ["query", "--in", "."];
    const CONSUMED_BY: &'static str = "--consumed-by";
    const EMPTY_TREE: &'static str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
    const CHANGED: [&'static str; 6] = ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"];
    const NAMED: [&'static str; 7] =
        ["diff", "--name-only", "--no-renames", "HEAD^1", "HEAD", "--", ":(glob)*/**/tests/**/*.rs"];

    pub(in super::super) fn asked(ports: &impl Ports, reach: &Reach) -> Result<Tested, Failure<'static>> {
        let invocation = Invocation::new("git", Self::NAMED);
        let named = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let files: Vec<String> = named.as_ref().parse::<Lines>().into_iter().flatten().collect();
        let consumed = files.first().map_or(Ok(Vec::new()), |_| reach.consumed(ports, &files))?;
        Ok(Tested(files.iter().chain(&consumed).filter_map(|path| Self::home(Path::new(path))).collect()))
    }

    fn consumed(ports: &impl Ports, lint: &Path, files: &[String]) -> Result<Vec<String>, Failure<'static>> {
        let asked = files.iter().flat_map(|file| [OsString::from(Self::CONSUMED_BY), OsString::from(file)]);
        let invocation = Invocation::new(lint, Self::QUERY.map(OsString::from).into_iter().chain(asked));
        let answer = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        Ok(answer.as_ref().parse::<Lines>().into_iter().flatten().collect())
    }

    fn home(path: &Path) -> Option<PathBuf> {
        path.ancestors()
            .find(|directory| Self::MARKS.iter().any(|mark| directory.file_name() == Some(OsStr::new(mark))))
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    }

    pub(in super::super) fn diff(&self, ports: &impl Ports) -> Result<Text, Failure<'static>> {
        let excluded = self.0.iter().map(|directory| Self::spec(":(exclude)", directory, "src"));
        let sources = self.0.iter().map(|directory| Self::spec(":(glob)", directory, "src/**/*.rs"));
        let whole = ["diff", "--no-renames", Self::EMPTY_TREE, "HEAD", "--"].map(OsString::from);
        let invocations = [
            Invocation::new("git", Self::CHANGED.map(OsString::from).into_iter().chain(excluded)),
            Invocation::new("git", whole.into_iter().chain(sources)),
        ];
        let answers = self.0.first().map_or(&invocations[..1], |_| &invocations[..]);
        let texts = Failure::started(ports.outputs(answers), answers)?;
        Ok(Text::new(texts.iter().map(AsRef::as_ref).collect::<Vec<&str>>().concat()))
    }

    fn spec(magic: &str, directory: &Path, below: &str) -> OsString {
        let mut spec = OsString::from(magic);
        spec.push(directory.join(below));
        spec
    }
}
