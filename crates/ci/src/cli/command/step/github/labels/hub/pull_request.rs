use text_lines::Lines;

use super::super::check::Check;
use crate::cli::ports::Text;

#[derive(Debug)]
pub struct PullRequest(Text);

impl PullRequest {
    pub(crate) const KIND: &'static str =
        r#"if (.title | startswith("refactor:")) then "refactor" else "change" end, .labels[].name"#;
    const REFACTOR_KIND: &'static str = "refactor";

    pub(crate) fn listed(kind_then_labels: Text) -> PullRequest {
        PullRequest(kind_then_labels)
    }

    pub(crate) fn carrying(&self, check: Check) -> Option<&PullRequest> {
        Lines::from(self.0.as_ref())
            .skip(1)
            .filter_map(|name| name.parse::<Check>().ok())
            .find(|carried| *carried == check)
            .map(|_| self)
    }

    pub(crate) fn refactor(&self) -> Option<&PullRequest> {
        Lines::from(self.0.as_ref()).next().filter(|kind| *kind == Self::REFACTOR_KIND).map(|_| self)
    }
}
