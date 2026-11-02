use text_lines::Lines;

#[derive(Debug)]
pub struct PullRequest(String);

impl PullRequest {
    pub(crate) const KIND: &'static str =
        r#"if (.title | startswith("refactor:")) then "refactor" else "change" end, .labels[].name"#;
    const REFACTOR_KIND: &'static str = "refactor";

    pub(crate) fn listed(kind_then_labels: String) -> PullRequest {
        PullRequest(kind_then_labels)
    }

    pub(crate) fn carrying(&self, label: &str) -> Option<&str> {
        Lines::from(self.0.as_str()).skip(1).find(|name| *name == label)
    }

    pub(crate) fn refactor(&self) -> Option<&str> {
        Lines::from(self.0.as_str()).next().filter(|kind| *kind == Self::REFACTOR_KIND)
    }
}
