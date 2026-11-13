mod command_paths;

pub use command_paths::{CommandPaths, Scope};
use graph::Model;
use lint_core::{Fix, Headline, Message, ModelLint, Problem, Report, Violation};

#[derive(Debug)]
pub struct ProcessStarters;

impl ModelLint for ProcessStarters {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("only ports start processes"),
            CommandPaths::queried(model, Scope::OutsidePorts)
                .into_iter()
                .map(|placed| {
                    Violation::new(
                        placed.site(),
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
