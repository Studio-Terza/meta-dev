use serde::Deserialize;
use text_lines::OneLine;

use super::Role;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(in super::super) enum Target {
    Fixed(Fixed),
    Named(OneLine),
}

#[derive(Debug, Deserialize)]
pub(in super::super) enum Fixed {
    #[serde(rename = "")]
    HostBuild,
    #[serde(rename = " (check)")]
    Library,
    #[serde(rename = " (check-test)")]
    LibraryTest,
    #[serde(rename = " build-script")]
    BuildScript,
    #[serde(rename = " build-script (run)")]
    BuildRun,
}

impl Role {
    const CHECK_TEST: &'static str = "(check-test)";

    fn named(target: OneLine) -> Role {
        let letters: Vec<char> = target.as_ref().chars().collect();
        let words: Vec<String> = letters
            .chunk_by(|left, right| left.is_whitespace() == right.is_whitespace())
            .filter_map(|chunk| chunk.first().filter(|letter| letter.is_whitespace()).map_or(Some(chunk), |_| None))
            .map(|chunk| chunk.iter().collect())
            .collect();
        let role: fn(OneLine) -> Role =
            words.last().filter(|last| *last == Self::CHECK_TEST).map_or(Role::Target, |_| Role::TestTarget);
        role(target)
    }
}

impl From<Target> for Role {
    fn from(target: Target) -> Role {
        match target {
            Target::Fixed(fixed) => Role::from(fixed),
            Target::Named(named) => Role::named(named),
        }
    }
}

impl From<Fixed> for Role {
    fn from(fixed: Fixed) -> Role {
        match fixed {
            Fixed::HostBuild => Role::HostBuild,
            Fixed::Library => Role::Library,
            Fixed::LibraryTest => Role::LibraryTest,
            Fixed::BuildScript => Role::BuildScript,
            Fixed::BuildRun => Role::BuildRun,
        }
    }
}
