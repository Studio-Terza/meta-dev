use text_lines::Lines;

use super::super::check::Check;
use super::super::fault::Fault;
use crate::cli::ports::Text;

#[derive(Debug)]
pub struct PullRequest(Text);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Refactor,
    Change,
}

impl PullRequest {
    pub(crate) const KIND: &'static str =
        r#"if (.title | startswith("refactor:")) then "refactor" else "change" end, .labels[].name"#;
    const REFACTOR_KIND: &'static str = "refactor";

    pub(crate) fn listed(kind_then_labels: Text) -> PullRequest {
        PullRequest(kind_then_labels)
    }

    pub(crate) fn carried(&self) -> Vec<Check> {
        self.0
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .skip(1)
            .filter_map(|name| name.parse::<Check>().ok())
            .collect()
    }

    pub(crate) fn purpose(&self) -> Purpose {
        self.0
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .next()
            .filter(|kind| *kind == Self::REFACTOR_KIND)
            .map_or(Purpose::Change, |_| Purpose::Refactor)
    }
}

impl Purpose {
    pub(crate) fn waiver(self) -> Option<Fault> {
        match self {
            Purpose::Refactor => Some(Fault::Refactor),
            Purpose::Change => None,
        }
    }
}
