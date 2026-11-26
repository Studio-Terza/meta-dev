mod command_paths;

pub use command_paths::{CommandPaths, Scope};
use lint_core::{Fix, Headline, Message, Problem, Site, SourceFile, Violation};
use lint_file::{FileLint, Reads};

#[derive(Debug)]
pub struct ProcessStarters;

impl FileLint for ProcessStarters {
    const HEADLINE: Headline = Headline::new("only ports start processes");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        CommandPaths::found(file, Scope::OutsidePorts).into_iter().map(ProcessStarters::flagged).collect()
    }
}

impl ProcessStarters {
    fn flagged(site: Site<'_>) -> Violation<'_> {
        Violation::new(
            site,
            Message::new(
                Problem::new("starts a std::process::Command"),
                Fix::new("move it behind a port: a crate whose name ends in ports, or a module named ports"),
            ),
        )
    }
}
