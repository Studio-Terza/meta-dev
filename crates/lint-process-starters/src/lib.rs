mod command_paths;

use code_model::Model;
pub use command_paths::{CommandPaths, Scope};
use lint_core::{Fix, Headline, Message, ModelLint, Problem, Report, Violation};

#[derive(Debug)]
pub struct ProcessStarters;

impl ModelLint for ProcessStarters {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("only ports start processes"),
            CommandPaths::across(model, Scope::OutsidePorts)
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
