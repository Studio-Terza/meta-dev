use lint_core::Violation;
use lint_layout::{PathTail, UsePath, Word, words};
use lint_paths::{Cover, Ground, Use};

use super::Finding;

#[derive(Debug)]
pub struct RawPaths;

impl RawPaths {
    pub const FINALS: [Word<'static>; 3] = words!["remove_dir_all", "forget", "temp_dir"];
    const RAW: [[Word<'static>; 2]; 3] =
        [words!["fs", "remove_dir_all"], words!["mem", "forget"], words!["env", "temp_dir"]];

    pub fn found<'file>(uses: &[Use<'file>]) -> Vec<Violation<'file>> {
        uses.iter()
            .filter_map(|used| {
                Self::raw(used).and_then(|path| Self::finding(used, path)).map(|finding| (used, finding))
            })
            .map(|(used, finding)| Violation::new(used.place(), finding))
            .collect()
    }

    fn raw<'used>(used: &'used Use<'_>) -> Option<&'used UsePath> {
        Self::RAW.iter().find_map(|tail| used.spelling(PathTail::new(tail)))
    }

    fn finding(used: &Use<'_>, path: &UsePath) -> Option<Finding> {
        match (used.ground(), used.cover()) {
            (Ground::Open, _) => Some(Finding::OutsidePorts(path.clone())),
            (Ground::Ports, Cover::Bare) => Some(Finding::OutsideGuard(path.clone())),
            (Ground::Ports, Cover::Guarded) => None,
        }
    }
}
