#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Rewrite,
    MoveToAssets,
}

impl From<Fix> for lint_core::Fix {
    fn from(fix: Fix) -> lint_core::Fix {
        lint_core::Fix::new(match fix {
            Fix::Rewrite => "rewrite it in Rust",
            Fix::MoveToAssets => {
                "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust"
            }
        })
    }
}
