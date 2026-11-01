mod command_paths;

pub use command_paths::{CommandPaths, Scope};
use lint_core::{Gate, Lint};
use lint_core::{Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct ProcessStarters;

impl Lint for ProcessStarters {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "only ports start processes",
            files
                .iter()
                .filter(|file| !Gate::PORTS.admits(file))
                .flat_map(|file| {
                    CommandPaths::found(file, Scope::OutsidePorts).into_iter().map(move |line| {
                        Violation::new(
                            Site::new(file.path(), line),
                            "starts a std::process::Command; move it behind a port: a crate whose name ends in ports, or a module named ports",
                        )
                    })
                })
                .collect(),
        )
    }
}
