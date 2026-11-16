use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use text_lines::Lines;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub(in super::super) struct Tested(BTreeSet<PathBuf>);

impl Tested {
    const TESTS: &'static str = "tests";
    const EMPTY_TREE: &'static str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
    const CHANGED: [&'static str; 6] = ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"];
    const NAMED: [&'static str; 7] =
        ["diff", "--name-only", "--no-renames", "HEAD^1", "HEAD", "--", ":(glob)*/**/tests/**/*.rs"];

    pub(in super::super) fn asked(ports: &impl Ports) -> Result<Tested, Failure<'static>> {
        let invocation = Invocation::new("git", Self::NAMED);
        let named = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let tested = named.as_ref().parse::<Lines>().into_iter().flatten().filter_map(|name| {
            Path::new(&name)
                .ancestors()
                .find(|directory| directory.file_name() == Some(OsStr::new(Self::TESTS)))
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        });
        Ok(Tested(tested.collect()))
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
