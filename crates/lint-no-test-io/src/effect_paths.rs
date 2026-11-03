use std::ffi::OsStr;
use std::path::Path;

use lint_core::{Gate, SourceFile};
use lint_layout::{PathTail, UsePath, Word, words};
use lint_paths::Use;
use lint_placement::Role;

#[derive(Debug)]
pub struct EffectPaths;

impl EffectPaths {
    const TESTS: &str = "tests";
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
    const WORLD: [&[Word<'static>]; 7] = [
        &words!["std", "io", "stdin"],
        &words!["std", "io", "stdout"],
        &words!["std", "io", "stderr"],
        &words!["file_ports", "Disk"],
        &words!["console_ports", "Process"],
        &words!["golden", "Disk"],
        &words!["golden", "Process"],
    ];
    const VALUES: [&[Word<'static>]; 1] = [&words!["std", "process", "ExitCode"]];

    pub fn tested(file: &SourceFile) -> Option<&SourceFile> {
        let placement = file.placement();
        (placement.role() == Role::Target
            && placement.crate_root().directory().parent().and_then(Path::file_name) == Some(OsStr::new(Self::TESTS))
            && Gate::PORTS.owns(placement).is_none())
        .then_some(file)
    }

    pub fn effect<'used>(used: &'used Use<'_>) -> Option<&'used UsePath> {
        Self::EFFECTS
            .iter()
            .find_map(|root| used.under(root))
            .filter(|_| Self::VALUES.iter().all(|value| used.under(value).is_none()))
            .or_else(|| Self::WORLD.iter().find_map(|world| Self::named(used, world)))
    }

    fn named<'used>(used: &'used Use, world: &[Word<'_>]) -> Option<&'used UsePath> {
        used.spelling(PathTail::new(world)).or_else(|| used.under(world))
    }
}
