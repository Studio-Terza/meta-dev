mod finding;

use lint_core::Violation;
use lint_layout::{PathTail, UsePath, Word, words};
use lint_paths::{Cover, Ground, Use};

use finding::Finding;

#[derive(Debug)]
pub struct RawPaths;

impl RawPaths {
    pub const FINALS: [Word<'static>; 3] = words!["remove_dir_all", "forget", "temp_dir"];
    const RAW: [[Word<'static>; 2]; 3] =
        [words!["fs", "remove_dir_all"], words!["mem", "forget"], words!["env", "temp_dir"]];

    pub fn found<'file>(uses: &[Use<'file>]) -> Vec<Violation<'file>> {
        uses.iter()
            .filter_map(|used| {
                Self::RAW
                    .iter()
                    .find_map(|tail| used.spelling(PathTail::new(tail)))
                    .and_then(|path| Self::finding(used, path))
                    .map(|finding| (used, finding))
            })
            .map(|(used, finding)| Violation::new(used.place(), finding))
            .collect()
    }

    fn finding<'path>(used: &Use<'_>, path: &'path UsePath) -> Option<Finding<'path>> {
        match (used.ground(), used.cover()) {
            (Ground::Open, _) => Some(Finding::OutsidePorts(path)),
            (Ground::Ports, Cover::Bare) => Some(Finding::OutsideGuard(path)),
            (Ground::Ports, Cover::Guarded) => None,
        }
    }
}
