use crate::ids::Minutes;
use crate::spec::{SelfFile, Unreadable};
use serde::Deserialize;

/// Per-role lease defaults from `self/roles.toml`, each with a source.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    work: Role,
    review: Role,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Role {
    deadline_minutes: Minutes,
    #[allow(dead_code)]
    source: String,
}

impl Roles {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        SelfFile::Roles.parse(text)
    }

    /// How long a work lease runs by default.
    pub fn work(&self) -> Minutes {
        self.work.deadline_minutes
    }

    /// How long a review lease runs by default.
    pub fn review(&self) -> Minutes {
        self.review.deadline_minutes
    }
}

#[cfg(test)]
mod tests {
    use super::Roles;

    #[test]
    fn the_repo_roles_give_work_90_and_review_30_minutes_with_the_owner_as_source() {
        let roles = Roles::parse(include_str!("../../../self/roles.toml")).unwrap();
        assert_eq!((roles.work().to_string(), roles.review().to_string()), ("90 min".to_owned(), "30 min".to_owned()));
        let source = "owner 2026-10-30: agents can and will stall";
        assert!(roles.work.source == source && roles.review.source == source);
    }

    #[test]
    fn roles_without_a_source_or_with_a_zero_deadline_do_not_parse() {
        let unsourced = "[work]\ndeadline-minutes = 90\n[review]\ndeadline-minutes = 30\nsource = \"s\"\n";
        let zero = "[work]\ndeadline-minutes = 0\nsource = \"s\"\n[review]\ndeadline-minutes = 30\nsource = \"s\"\n";
        assert!(Roles::parse(unsourced).is_err() && Roles::parse(zero).is_err());
    }
}
