use lint_layout::{PathTail, UsePath, Word, words};
use lint_paths::Use;

#[derive(Debug)]
pub struct EffectPaths;

impl EffectPaths {
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

    pub fn effect(used: &Use) -> Option<&UsePath> {
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
