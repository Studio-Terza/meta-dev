mod command_paths;

use crate::Lint;
use command_paths::CommandPaths;
use lint_core::{Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct ProcessStarters;

impl Lint for ProcessStarters {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "linted code does not start processes",
            files
                .iter()
                .filter(|file| file.is_shipped())
                .flat_map(|file| {
                    CommandPaths::found(file).into_iter().map(move |line| {
                        Violation::new(
                            Site::new(file.path(), line),
                            "starts a std::process::Command; start processes through a runner outside the linted code",
                        )
                    })
                })
                .collect(),
        )
    }
}
