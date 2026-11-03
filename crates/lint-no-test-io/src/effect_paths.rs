use std::ffi::OsStr;
use std::path::Path;

use lint_core::SourceFile;
use lint_layout::{UsePath, Word, words};
use lint_paths::Use;
use lint_placement::Role;

#[derive(Debug)]
pub struct EffectPaths;

impl EffectPaths {
    const TESTS: &str = "tests";
    const PORTS: &str = "ports";
    const EFFECTS: [&[Word<'static>]; 8] = [
        &words!["std", "fs"],
        &words!["std", "net"],
        &words!["std", "env"],
        &words!["std", "process"],
        &words!["std", "os", "unix", "fs"],
        &words!["std", "os", "unix", "net"],
        &words!["std", "os", "windows", "fs"],
        &words!["tempfile"],
    ];
    const VALUES: [&[Word<'static>]; 1] = [&words!["std", "process", "ExitCode"]];

    pub fn tested(file: &SourceFile) -> Option<&SourceFile> {
        let placement = file.placement();
        let target = placement.crate_root().directory();
        (placement.role() == Role::Target
            && target.parent().and_then(Path::file_name) == Some(OsStr::new(Self::TESTS))
            && target.file_stem() != Some(OsStr::new(Self::PORTS)))
        .then_some(file)
    }

    pub fn effect(used: &Use) -> Option<&UsePath> {
        Self::EFFECTS
            .iter()
            .find_map(|root| used.under(root))
            .filter(|_| Self::VALUES.iter().all(|value| used.under(value).is_none()))
    }
}
