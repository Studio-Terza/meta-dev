#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Rewrite,
    MoveToAssets,
}

impl From<Fix> for lint_core::Fix {
    fn from(fix: Fix) -> lint_core::Fix {
        match fix {
            Fix::Rewrite => lint_core::Fix::new("rewrite it in Rust"),
            Fix::MoveToAssets => lint_core::Fix::new(
                "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust",
            ),
        }
    }
}
