use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use text_lines::Lines;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub(in super::super) struct Tested {
    homes: BTreeSet<PathBuf>,
    base: OsString,
}

impl Tested {
    const MARKS: [&'static str; 2] = ["src", "tests"];
    const EMPTY_TREE: &'static str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
    const CHANGED: [&'static str; 2] = ["diff", "--no-renames"];
    const CHANGED_PATHS: [&'static str; 3] = ["HEAD", "--", "*.rs"];
    const NAMED: [&'static str; 3] = ["diff", "--name-only", "--no-renames"];
    const NAMED_PATHS: [&'static str; 2] = ["--", ":(glob)*/**/tests/**/*.rs"];
    pub(in super::super) const HEAD: &'static str = "HEAD";
    pub(in super::super) const PULL_BASE: &'static str = "HEAD^1";

    pub(in super::super) fn asked(ports: &impl Ports, base: &OsStr, head: &OsStr) -> Result<Tested, Failure<'static>> {
        let mut arguments = Self::around(&Self::NAMED, base, &[]);
        arguments.push(head.to_owned());
        arguments.extend(Self::NAMED_PATHS.map(OsString::from));
        let invocation = Invocation::new("git", arguments);
        let named = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let files: Vec<String> = named.as_ref().parse::<Lines>().into_iter().flatten().collect();
        let homes = files.iter().filter_map(|path| Self::home(Path::new(path))).collect();
        Ok(Tested { homes, base: base.to_owned() })
    }

    pub(in super::super) fn without(self, earlier: &Tested) -> Tested {
        Tested { homes: self.homes.difference(&earlier.homes).cloned().collect(), ..self }
    }

    fn home(path: &Path) -> Option<PathBuf> {
        path.ancestors()
            .find(|directory| Self::MARKS.iter().any(|mark| directory.file_name() == Some(OsStr::new(mark))))
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    }

    pub(in super::super) fn diff(&self, ports: &impl Ports) -> Result<Text, Failure<'static>> {
        let excluded = self.homes.iter().map(|directory| Self::spec(":(exclude)", directory, "src"));
        let sources = self.homes.iter().map(|directory| Self::spec(":(glob)", directory, "src/**/*.rs"));
        let whole = ["diff", "--no-renames", Self::EMPTY_TREE, "HEAD", "--"].map(OsString::from);
        let invocations = [
            Invocation::new(
                "git",
                Self::around(&Self::CHANGED, &self.base, &Self::CHANGED_PATHS).into_iter().chain(excluded),
            ),
            Invocation::new("git", whole.into_iter().chain(sources)),
        ];
        let answers = self.homes.first().map_or(&invocations[..1], |_| &invocations[..]);
        let texts = Failure::started(ports.outputs(answers), answers)?;
        Ok(Text::new(texts.iter().map(AsRef::as_ref).collect::<Vec<&str>>().concat()))
    }

    fn around(before: &[&str], base: &OsStr, after: &[&str]) -> Vec<OsString> {
        before.iter().map(OsString::from).chain([base.to_owned()]).chain(after.iter().map(OsString::from)).collect()
    }

    fn spec(magic: &str, directory: &Path, below: &str) -> OsString {
        let mut spec = OsString::from(magic);
        spec.push(directory.join(below));
        spec
    }
}
