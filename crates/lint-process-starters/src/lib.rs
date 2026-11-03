mod command_paths;

pub use command_paths::{CommandPaths, Scope};
use lint_core::Lint;
use lint_core::{Fix, Headline, Message, Problem, Report, SourceFile, Violation};

#[derive(Debug)]
pub struct ProcessStarters;

impl Lint for ProcessStarters {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("only ports start processes"),
            CommandPaths::across(files, Scope::OutsidePorts)
                .into_iter()
                .map(|site| {
                    Violation::new(
                        site,
                        Message::new(
                            Problem::new("starts a std::process::Command"),
                            Fix::new(
                                "move it behind a port: a crate whose name ends in ports, or a module named ports",
                            ),
                        ),
                    )
                })
                .collect(),
        )
    }
}
