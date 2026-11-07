#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Rewrite,
    MoveToAssets,
}

impl From<Fix> for lint_core::Fix {
    fn from(fix: Fix) -> Self {
        match fix {
            Fix::Rewrite => Self::new("rewrite it in Rust"),
            Fix::MoveToAssets => {
                Self::new("if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust")
            }
        }
    }
}
